use anyhow::{bail, Result};
use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  BufferSize, SampleRate, StreamConfig,
};
use parking_lot::{Condvar, Mutex};
use rb::*;
use std::{
  fs::File,
  path::{Path, PathBuf},
  sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
  sync::Arc,
  thread,
  time::{self, Instant},
};
use symphonia::core::{
  audio::{AudioBufferRef, RawSample, SampleBuffer, SignalSpec},
  codecs::{DecoderOptions, CODEC_TYPE_NULL},
  conv::ConvertibleSample,
  errors::Error,
  formats::{FormatOptions, FormatReader, Track},
  io::MediaSourceStream,
  meta::MetadataOptions,
  probe::Hint,
  units::{Duration, Time},
};

trait AudioOutputSample:
  cpal::Sample + ConvertibleSample + RawSample + std::marker::Send + 'static
{
}
impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}

pub struct Symphonia {
  duration: u64,
  last_played: Option<PathBuf>,
  controls: Arc<Controls>,
}

#[derive(Default)]
struct Controls {
  position: AtomicU64,
  is_paused: (Mutex<bool>, Condvar),
  track_finished: AtomicBool,
  seek_to: AtomicU32,
}

trait SymphoniaReader {
  fn default_track(&self) -> &Track;
}
impl SymphoniaReader for Box<dyn FormatReader> {
  fn default_track(&self) -> &Track {
    self
      .tracks()
      .iter()
      .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
      .expect("No supported audio tracks")
  }
}

fn reset_audio_output(
  output: &mut Option<Box<dyn AudioOutput>>,
  spec: &Option<SignalSpec>,
  duration: u64,
) {
  if let Some(spec) = spec {
    output.replace(try_open(*spec, duration).unwrap());
  }
}

impl Symphonia {
  fn get_reader(path: &Path) -> Result<Box<dyn FormatReader>> {
    let src = File::open(path)?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let hint = Hint::new();
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;

    Ok(probed.format)
  }
}

impl super::Backend for Symphonia {
  fn new() -> Self {
    Self {
      duration: 0,
      controls: Arc::new(Controls::default()),
      last_played: None,
    }
  }

  fn last_played(&self) -> Option<&PathBuf> {
    self.last_played.as_ref()
  }

  fn play(&mut self, path: Option<&Path>) -> Result<()> {
    self.controls = Arc::new(Controls::default());

    if let Some(path) = path {
      self.last_played = Some(path.to_owned());

      let mut reader = Self::get_reader(path)?;
      let track = reader.default_track();

      let tb = track.codec_params.time_base.unwrap();
      let duration = track
        .codec_params
        .n_frames
        .map(|f| tb.calc_time(track.codec_params.start_ts + f).seconds);

      self.duration = duration.unwrap_or(0);

      let decoder_options = DecoderOptions::default();

      thread::spawn({
        let controls = self.controls.clone();
        let track_id = track.id;
        let mut decoder =
          symphonia::default::get_codecs().make(&track.codec_params, &decoder_options)?;

        move || {
          let mut audio_output: Option<Box<dyn AudioOutput>> = None;
          let mut last_pos_update = Instant::now();
          let mut spec: Option<SignalSpec> = None;
          let mut duration = 0;

          // If controls is not referenced elsewhere, it means the main app has moved to a different track.
          while Arc::strong_count(&controls) != 1 {
            // pausing
            let &(ref lock, ref cvar) = &controls.is_paused;
            let mut is_paused = lock.lock();
            if *is_paused {
              audio_output.as_ref().map(|ao| ao.pause());
              cvar.wait(&mut is_paused);
              reset_audio_output(&mut audio_output, &spec, duration);
              audio_output.as_ref().map(|ao| ao.play());
            }

            // seeking
            let seek_to = controls.seek_to.swap(0, Ordering::SeqCst);
            if seek_to != 0 {
              let _ = reader.seek(
                symphonia::core::formats::SeekMode::Accurate,
                symphonia::core::formats::SeekTo::Time {
                  time: Time::from(seek_to),
                  track_id: None,
                },
              );
            }

            let packet = match reader.next_packet() {
              Ok(packet) => packet,
              Err(_) => break,
            };

            if packet.track_id() != track_id {
              continue;
            }

            // Update position
            if last_pos_update.elapsed() >= time::Duration::from_secs(1) {
              let secs = tb.calc_time(packet.ts()).seconds;
              controls.position.store(secs, Ordering::SeqCst);
              last_pos_update = Instant::now();
            }

            match decoder.decode(&packet) {
              Ok(decoded) => {
                if audio_output.is_none() {
                  spec = Some(*decoded.spec());
                  duration = decoded.capacity() as Duration;
                  reset_audio_output(&mut audio_output, &spec, duration);
                  // audio_output.replace(try_open(spec.unwrap(), duration).unwrap());
                }

                if let Some(audio_output) = &mut audio_output {
                  audio_output.write(decoded).unwrap();
                }
              }
              Err(Error::DecodeError(err)) => {
                println!("decode error: {}", err);
              }
              _ => {
                break;
              }
            }
          }

          // wait for the buffer to flush
          thread::sleep(time::Duration::from_millis(200));

          controls.track_finished.store(true, Ordering::SeqCst);
        }
      });
    }

    Ok(())
  }

  fn track_finished(&self) -> bool {
    self.controls.track_finished.load(Ordering::Relaxed)
  }

  fn is_paused(&self) -> bool {
    *self.controls.is_paused.0.lock()
  }

  fn pause(&mut self) {
    let &(ref lock, ref cvar) = &self.controls.is_paused;
    *lock.lock() = true;
    cvar.notify_one();
  }

  fn play_pause(&mut self) {
    let &(ref lock, ref cvar) = &self.controls.is_paused;
    let mut is_paused = lock.lock();
    *is_paused = !*is_paused;
    cvar.notify_one();
  }

  // (pct, pos, dur)
  fn progress(&self) -> (f64, u64, u64) {
    let duration = self.duration;
    let position = self.controls.position.load(Ordering::SeqCst);
    ((position as f64 / duration as f64), position, duration)
  }

  fn seek(&mut self, time: u64) {
    self.controls.position.store(time, Ordering::SeqCst);
    self.controls.seek_to.store(time as u32, Ordering::SeqCst);
  }
  fn seek_delta(&mut self, delta_time: i64) {
    let position = self.controls.position.load(Ordering::Relaxed) as i64;
    self.seek(position.saturating_add(delta_time) as u64);
  }
}

struct CpalAudioOutputImpl<T: AudioOutputSample>
where
  T: AudioOutputSample,
{
  ring_buf_tx: rb::Producer<T>,
  sample_buf: SampleBuffer<T>,
  stream: cpal::Stream,
}

pub trait AudioOutput {
  fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()>;
  fn flush(&mut self);
  fn pause(&self);
  fn play(&self);
}

fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn AudioOutput>> {
  let host = cpal::default_host();
  let device = match host.default_output_device() {
    Some(device) => device,
    _ => {
      bail!("failed to get default output device");
    }
  };
  let config = match device.default_output_config() {
    Ok(config) => config,
    Err(err) => {
      bail!("failed to get default audio output device config: {}", err);
    }
  };

  match config.sample_format() {
    cpal::SampleFormat::F32 => CpalAudioOutputImpl::<f32>::try_open(spec, duration, &device),
    cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::try_open(spec, duration, &device),
    cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::try_open(spec, duration, &device),
    _ => unreachable!(), // We shouldn't reach here... right?
  }
}

impl<T: AudioOutputSample + cpal::SizedSample> CpalAudioOutputImpl<T> {
  pub fn try_open(
    spec: SignalSpec,
    duration: Duration,
    device: &cpal::Device,
  ) -> Result<Box<dyn AudioOutput>> {
    let channels = spec.channels.count() as usize;

    let sample_buf = SampleBuffer::<T>::new(duration, spec);
    let config = StreamConfig {
      channels: channels as cpal::ChannelCount,
      sample_rate: SampleRate(spec.rate),
      buffer_size: BufferSize::Default,
    };

    let ring_len = ((200 * spec.rate as usize) / 1000) * channels;
    let ring_buf = SpscRb::new(ring_len);
    let (ring_buf_tx, ring_buf_rx) = (ring_buf.producer(), ring_buf.consumer());

    let stream = device.build_output_stream(
      &config,
      move |data: &mut [T], _| {
        let written = ring_buf_rx.read(data).unwrap_or(0);
        data[written..].iter_mut().for_each(|s| *s = T::MID);
      },
      move |_| {},
      None,
    )?;

    stream.play()?;

    Ok(Box::new(Self {
      ring_buf_tx,
      sample_buf,
      stream,
    }))
  }
}

impl<T: AudioOutputSample> AudioOutput for CpalAudioOutputImpl<T> {
  fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {
    // Do nothing if there are no audio frames.
    if decoded.frames() == 0 {
      return Ok(());
    }

    // Audio samples must be interleaved for cpal. Interleave the samples in the audio
    // buffer into the sample buffer.
    self.sample_buf.copy_interleaved_ref(decoded);

    // Write all the interleaved samples to the ring buffer.
    let mut samples = self.sample_buf.samples();

    while let Some(written) = self.ring_buf_tx.write_blocking(samples) {
      samples = &samples[written..];
    }

    Ok(())
  }

  fn flush(&mut self) {
    // Flush is best-effort, ignore the returned result.
    let _ = self.stream.pause();
  }

  fn pause(&self) {
    let _ = self.stream.pause();
  }
  fn play(&self) {
    let _ = self.stream.play();
  }
}
