pub use anyhow::{bail, Result};
pub use crossbeam_channel::{Receiver, Sender};
pub use hashbrown::{HashMap, HashSet};
pub use parking_lot::Mutex;
pub use std::{
  boxed::Box,
  ffi::OsStr,
  fs::{self, File},
  io::BufReader,
  ops::Range,
  path::{Path, PathBuf},
  rc::Rc,
  sync::Arc,
  thread,
  time::Duration,
};

pub const SUPPORTED: &'static [&'static str] =
  &["mp3", "ogg", "opus", "flac", "wav", "webm", "mp4"];

pub fn extension<'a>(path: &'a Path) -> Option<&'a str> {
  if let Some(ext) = path.extension() {
    return ext.to_str();
  }
  None
}

pub trait AquinasVec<T> {
  fn get_range(&self, range: &Range<usize>) -> &[T];
}

impl<T> AquinasVec<T> for Vec<T> {
  fn get_range(&self, range: &Range<usize>) -> &[T] {
    &self[range.start.min(self.len())..range.end.min(self.len())]
  }
}
