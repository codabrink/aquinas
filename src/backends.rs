#[cfg(feature = "gstreamer_backend")]
mod gstreamer_backend;
#[cfg(feature = "symphonia_backend")]
mod symphonia_backend;

use std::boxed::Box;
use std::path::{Path, PathBuf};

pub fn load() -> Box<dyn Backend> {
  // in the future this will be configurable,
  // but for now we only have one backend.
  #[cfg(feature = "gstreamer_backend")]
  return Box::new(gstreamer_backend::GStreamer::new());
  #[cfg(feature = "symphonia_backend")]
  return Box::new(symphonia_backend::Symphonia::new());
}

pub trait Backend {
  fn new() -> Self
  where
    Self: Sized;
  fn track_finished(&self) -> bool;
  fn play(&mut self, path: Option<&Path>) -> anyhow::Result<()>;
  fn pause(&mut self);
  fn is_paused(&self) -> bool;
  fn last_played(&self) -> Option<&PathBuf>;
  fn play_pause(&mut self);
  fn seek(&mut self, time: u64);
  fn seek_delta(&mut self, delta_time: i64);
  fn progress(&self) -> (f64, u64, u64); // (pct, pos, dur)
}
