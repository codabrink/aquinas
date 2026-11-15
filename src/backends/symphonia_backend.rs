use anyhow::{bail, Result};
use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  BufferSize, StreamConfig,
};
use parking_lot::{Condvar, Mutex};
use rb::*;
use rubato::{
  audioadapter_buffers::direct::InterleavedSlice, Fft, FixedSync, Resampler as _,
};
use std::{
  fs::File,
  path::{Path, PathBuf},
  sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
  sync::Arc,
  thread,
  time::{self, Instant},
};
use symphonia::core::{
  audio::{conv::ConvertibleSample, AudioSpec, GenericAudioBufferRef},
  codecs::audio::AudioDecoderOptions,
  errors::Error,
  formats::{probe::Hint, FormatOptions, FormatReader, TrackType},
  io::MediaSourceStream,
  meta::MetadataOptions,
  units::{Time, Timestamp},
};

trait AudioOutputSample:
  cpal::Sample + cpal::FromSample<f32> + ConvertibleSample + std::marker::Send + 'static
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

fn reset_audio_output(
  output: &mut Option<Box<dyn AudioOutput>>,
  spec: &Option<AudioSpec>,
) {
  if let Some(spec) = spec {
    output.replace(try_open(spec).unwrap());
  }
}

impl Symphonia {
  fn get_reader(path: &Path) -> Result<Box<dyn FormatReader>> {
    let src = File::open(path)?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let hint = Hint::new();
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    let format = symphonia::default::get_probe().probe(&hint, mss, fmt_opts, meta_opts)?;

    Ok(format)
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
      let track = match reader.default_track(TrackType::Audio) {
        Some(track) => track,
        None => bail!("No supported audio tracks"),
      };

      let tb = track.time_base.unwrap();
      let duration = track
        .num_frames
        .and_then(|f| tb.calc_time(Timestamp::new(track.start_ts.get() + f as i64)))
        .map(|t| t.as_secs() as u64);

      self.duration = duration.unwrap_or(0);

      let decoder_options = AudioDecoderOptions::default();

      thread::spawn({
        let controls = self.controls.clone();
        let track_id = track.id;
        let codec_params = match track.codec_params.as_ref().and_then(|p| p.audio()) {
          Some(params) => params,
          None => bail!("track has no audio codec parameters"),
        };
        let mut decoder =
          symphonia::default::get_codecs().make_audio_decoder(codec_params, &decoder_options)?;

        move || {
          let mut audio_output: Option<Box<dyn AudioOutput>> = None;
          let mut last_pos_update = Instant::now();
          let mut spec: Option<AudioSpec> = None;

          // If controls is not referenced elsewhere, it means the main app has moved to a different track.
          while Arc::strong_count(&controls) != 1 {
            // pausing
            // Scope the guard so the lock isn't held while blocking on audio output,
            // otherwise the UI thread deadlocks on pause()/is_paused().
            {
              let &(ref lock, ref cvar) = &controls.is_paused;
              let mut is_paused = lock.lock();
              if *is_paused {
                audio_output.as_ref().map(|ao| ao.pause());
                cvar.wait(&mut is_paused);
                drop(is_paused);
                reset_audio_output(&mut audio_output, &spec);
                audio_output.as_ref().map(|ao| ao.play());
              }
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
              Ok(Some(packet)) => packet,
              _ => break,
            };

            if packet.track_id != track_id {
              continue;
            }

            // Update position
            if last_pos_update.elapsed() >= time::Duration::from_secs(1) {
              if let Some(time) = tb.calc_time(packet.pts) {
                controls.position.store(time.as_secs() as u64, Ordering::SeqCst);
              }
              last_pos_update = Instant::now();
            }

            match decoder.decode(&packet) {
              Ok(decoded) => {
                if audio_output.is_none() {
                  spec = Some(decoded.spec().clone());
                  reset_audio_output(&mut audio_output, &spec);
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
  sample_buf: Vec<T>,
  stream: cpal::Stream,
  rate_converter: Option<RateConverter>,
}

// Resamples to the device's rate when it can't open a stream at the file's rate
// (e.g. 48kHz opus on a 44.1kHz-only output).
struct RateConverter {
  resampler: Fft<f32>,
  channels: usize,
  decoded: Vec<f32>,
  pending: Vec<f32>,
  output: Vec<f32>,
}

impl RateConverter {
  const CHUNK_FRAMES: usize = 1024;

  fn new(rate_in: u32, rate_out: u32, channels: usize) -> Result<Self> {
    let resampler = Fft::new(
      rate_in as usize,
      rate_out as usize,
      Self::CHUNK_FRAMES,
      channels,
      FixedSync::Input,
    )?;
    Ok(Self {
      resampler,
      channels,
      decoded: Vec::new(),
      pending: Vec::new(),
      output: Vec::new(),
    })
  }

  // Buffers interleaved input and resamples every complete chunk.
  fn process(&mut self, decoded: &GenericAudioBufferRef<'_>) -> Result<&[f32]> {
    decoded.copy_to_vec_interleaved(&mut self.decoded);
    self.pending.extend_from_slice(&self.decoded);
    self.output.clear();

    let ch = self.channels;
    let mut consumed = 0;
    loop {
      let frames_in = self.resampler.input_frames_next();
      let chunk = frames_in * ch;
      if self.pending.len() - consumed < chunk {
        break;
      }

      let input = InterleavedSlice::new(&self.pending[consumed..consumed + chunk], ch, frames_in)?;
      let start = self.output.len();
      let frames_max = self.resampler.output_frames_max();
      self.output.resize(start + frames_max * ch, 0.0);
      let mut out = InterleavedSlice::new_mut(&mut self.output[start..], ch, frames_max)?;
      let (_, written) = self.resampler.process_into_buffer(&input, &mut out, None)?;
      self.output.truncate(start + written * ch);
      consumed += chunk;
    }
    self.pending.drain(..consumed);

    Ok(&self.output)
  }
}

pub trait AudioOutput {
  fn write(&mut self, decoded: GenericAudioBufferRef<'_>) -> Result<()>;
  fn flush(&mut self);
  fn pause(&self);
  fn play(&self);
}

fn try_open(spec: &AudioSpec) -> Result<Box<dyn AudioOutput>> {
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

  // Use the file's rate if the device supports it, otherwise fall back to the device's default.
  let channels = spec.channels().count() as cpal::ChannelCount;
  let rate_supported = device
    .supported_output_configs()
    .map(|mut configs| {
      configs.any(|c| {
        c.channels() == channels
          && c.min_sample_rate() <= spec.rate()
          && spec.rate() <= c.max_sample_rate()
      })
    })
    .unwrap_or(false);
  let output_rate = if rate_supported { spec.rate() } else { config.sample_rate() };

  match config.sample_format() {
    cpal::SampleFormat::F32 => CpalAudioOutputImpl::<f32>::try_open(spec, output_rate, &device),
    cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::try_open(spec, output_rate, &device),
    cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::try_open(spec, output_rate, &device),
    _ => unreachable!(), // We shouldn't reach here... right?
  }
}

impl<T: AudioOutputSample + cpal::SizedSample> CpalAudioOutputImpl<T> {
  pub fn try_open(
    spec: &AudioSpec,
    output_rate: u32,
    device: &cpal::Device,
  ) -> Result<Box<dyn AudioOutput>> {
    let channels = spec.channels().count();

    let sample_buf = Vec::new();
    let config = StreamConfig {
      channels: channels as cpal::ChannelCount,
      sample_rate: output_rate,
      buffer_size: BufferSize::Default,
    };

    let rate_converter = match output_rate == spec.rate() {
      true => None,
      false => Some(RateConverter::new(spec.rate(), output_rate, channels)?),
    };

    let ring_len = ((200 * output_rate as usize) / 1000) * channels;
    let ring_buf = SpscRb::new(ring_len);
    let (ring_buf_tx, ring_buf_rx) = (ring_buf.producer(), ring_buf.consumer());

    let stream = device.build_output_stream(
      config,
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
      rate_converter,
    }))
  }
}

impl<T: AudioOutputSample> AudioOutput for CpalAudioOutputImpl<T> {
  fn write(&mut self, decoded: GenericAudioBufferRef<'_>) -> Result<()> {
    // Do nothing if there are no audio frames.
    if decoded.frames() == 0 {
      return Ok(());
    }

    // Audio samples must be interleaved for cpal. Interleave the samples in the audio
    // buffer into the sample buffer, resampling first if needed.
    match &mut self.rate_converter {
      Some(converter) => {
        let resampled = converter.process(&decoded)?;
        self.sample_buf.clear();
        self.sample_buf.extend(resampled.iter().map(|&s| <T as cpal::Sample>::from_sample(s)));
      }
      None => decoded.copy_to_vec_interleaved(&mut self.sample_buf),
    }

    // Write all the interleaved samples to the ring buffer.
    let mut samples = &self.sample_buf[..];

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
