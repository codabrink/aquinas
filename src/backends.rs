mod gstreamer;
mod rodio;

use std::boxed::Box;
use std::path::{Path, PathBuf};

pub fn load() -> Box<dyn Backend> {
  // in the future this will be configurable,
  // but for now we only have one backend.

  // Box::new(gstreamer::GStreamer::new())
  Box::new(rodio::Rodio::new())
}

pub trait Backend {
  fn new() -> Self
  where
    Self: Sized;
  fn duration(path: &Path) -> u64
  where
    Self: Sized;
  fn play(&mut self, path: &Path);
  fn pause(&mut self);
  fn is_paused(&self) -> bool;
  fn last_played(&self) -> Option<&PathBuf>;
  fn toggle(&mut self);
  fn seek(&mut self, time: u64);
  fn seek_delta(&mut self, delta_time: i64);
  fn progress(&self) -> (f64, u64, u64); // (pct, pos, dur)
}
