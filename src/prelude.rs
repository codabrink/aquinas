pub use anyhow::{bail, Result};
pub use hashbrown::{HashMap, HashSet};
pub use parking_lot::Mutex;
pub use std::{
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

pub const SUPPORTED: &'static [&'static str] = &["mp3", "ogg", "opus", "flac", "wav"];

pub fn extension<'a>(path: &'a Path) -> Option<&'a str> {
  if let Some(ext) = path.extension() {
    return ext.to_str();
  }
  None
}
