mod file_iter;
mod file_tree;

pub use anyhow::{bail, Result};
pub use hashbrown::{HashMap, HashSet};
pub use std::{
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
  thread,
  time::Duration,
};

pub use crate::backends::Backend;
pub use file_tree::*;
pub const SUPPORTED: &'static [&'static str] = &["mp3", "ogg", "flac", "wav"];
