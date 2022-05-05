use crate::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::{
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
  sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

pub struct Rodio {
  _stream: OutputStream,
  stream: OutputStreamHandle,
  playing: Option<PathBuf>,
  playing_duration: u64,
  controls: Arc<Controls>,
}

#[derive(Default)]
struct Controls {
  paused: AtomicBool,

  // used to start playback in middle of file
  seek: AtomicU64,
  // used to skip forward without restarting playback
  seek_delta: AtomicU64,

  // cursor position = elapsed + instant
  cursor_elapsed: Mutex<Duration>,
  cursor_instant: Mutex<Option<Instant>>,
  // used to know when a new stream is playing so the current stream can stop
  playing_index: AtomicUsize,
}

impl Rodio {
  fn create_source(path: &Path) -> Option<Decoder<BufReader<File>>> {
    if let Ok(file) = File::open(path) {
      return Decoder::new(BufReader::new(file)).ok();
    }
    None
  }
}

impl super::Backend for Rodio {
  fn new() -> Self {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    Self {
      _stream: stream,
      stream: stream_handle,
      playing: None,
      playing_duration: 0,
      controls: Arc::new(Controls::default()),
    }
  }

  fn duration(path: &Path) -> u64 {
    crate::duration::duration(path).unwrap_or(0)
  }

  fn play(&mut self, path: &Path) {
    if let Some(source) = Self::create_source(path) {
      self.playing = Some(path.to_path_buf());
      let controls = self.controls.clone();
      let seek = self.controls.seek.swap(0, Ordering::SeqCst);

      let playing_index = controls.playing_index.fetch_add(1, Ordering::SeqCst) + 1;

      let source =
        source
          .pausable(false)
          .stoppable()
          .periodic_access(Duration::from_millis(5), move |src| {
            if playing_index != controls.playing_index.load(Ordering::SeqCst) {
              src.stop();
            }

            let paused = controls.paused.load(Ordering::SeqCst);
            src.inner_mut().set_paused(paused);

            let mut cursor_instant = controls.cursor_instant.lock();
            let mut cursor_elapsed = controls.cursor_elapsed.lock();

            match (paused, *cursor_instant) {
              (true, Some(ps)) => {
                *cursor_elapsed += ps.elapsed();
                *cursor_instant = None;
              }
              (false, None) => *cursor_instant = Some(Instant::now()),
              _ => {}
            };
          });

      let samples = source
        .skip_duration(Duration::from_secs(seek))
        .convert_samples();

      {
        *self.controls.cursor_instant.lock() = Some(Instant::now());
        *self.controls.cursor_elapsed.lock() = Duration::from_secs(seek);
        self.playing_duration = Self::duration(path);
      }

      let _ = self.stream.play_raw(samples);
    }
  }

  fn pause(&mut self) {
    self.controls.paused.store(true, Ordering::SeqCst);
  }

  fn is_paused(&self) -> bool {
    self.controls.paused.load(Ordering::SeqCst)
  }

  fn toggle(&mut self) {
    self.controls.paused.swap(
      !self.controls.paused.load(Ordering::SeqCst),
      Ordering::SeqCst,
    );
  }

  fn seek(&mut self, time: u64) {
    if let Some(playing_path) = &self.playing {
      self.controls.seek.store(time, Ordering::SeqCst);
      let playing_path = playing_path.clone();
      self.play(&playing_path);
    }
  }

  fn seek_delta(&mut self, delta_time: i64) {
    let position = (self.position() as i64).saturating_add(delta_time);
    self.seek(position as u64);
  }

  fn last_played(&self) -> Option<&PathBuf> {
    self.playing.as_ref()
  }

  fn progress(&self) -> (f64, u64, u64) {
    let position = self.position();
    let duration = self.playing_duration + 10;
    let percent = position as f64 / (duration as f64);
    (percent, position, duration)
  }
}

impl Rodio {
  fn position(&self) -> u64 {
    self.controls.cursor_elapsed.lock().as_secs()
      + match *self.controls.cursor_instant.lock() {
        Some(instant) => instant.elapsed().as_secs(),
        None => 0,
      }
  }
}
