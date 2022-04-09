use rodio::OutputStreamHandle;
use rodio::{Decoder, OutputStream, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
  seek: AtomicU64,

  playing_since: Mutex<Option<Instant>>,
  cursor: Mutex<Duration>,
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
    if let Some(source) = Self::create_source(path) {
      if let Some(duration) = source.total_duration() {
        return duration.as_secs();
      }
    }
    0
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

            let mut cursor = controls.cursor.lock().unwrap();
            let mut playing_since = controls.playing_since.lock().unwrap();

            match (paused, *playing_since) {
              (true, Some(ps)) => {
                *cursor += ps.elapsed();
                *playing_since = None;
              }
              (false, None) => *playing_since = Some(Instant::now()),
              _ => {}
            };
          });

      let samples = source
        .skip_duration(Duration::from_secs(seek))
        .convert_samples();

      *self.controls.playing_since.lock().unwrap() = Some(Instant::now());
      *self.controls.cursor.lock().unwrap() = Duration::from_secs(seek);
      self.playing_duration = Self::duration(path);

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

  fn seek_delta(&mut self, delta_time: i64) {}

  fn last_played(&self) -> Option<&PathBuf> {
    self.playing.as_ref()
  }

  fn progress(&self) -> (f64, u64, u64) {
    // return (0., 0, 0);
    let mut time_pos = self.controls.cursor.lock().unwrap().as_secs();
    let playing_since = self.controls.playing_since.lock().unwrap();

    if let Some(playing_since) = &*self.controls.playing_since.lock().unwrap() {
      time_pos += playing_since.elapsed().as_secs();
    }

    let duration = self.playing_duration;
    let percent = time_pos as f64 / (duration as f64);
    (percent, time_pos, duration)
  }
}

#[cfg(test)]
mod tests {
  use crate::backends::rodio::*;
  use crate::*;
  #[test]
  fn test_duration() {
    let path = Path::new("assets/brighter.ogg");

    assert!(path.is_file());
    let dur = Rodio::duration(path);
    assert!(dur > 0);
  }
}
