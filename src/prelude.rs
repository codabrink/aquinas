pub use anyhow::{bail, Result};
pub use hashbrown::{HashMap, HashSet};
pub use std::{
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
  rc::Rc,
  thread,
  time::Duration,
};

pub const SUPPORTED: &'static [&'static str] = &["mp3", "ogg", "opus", "flac", "wav"];
