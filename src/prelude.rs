pub mod file_iter;
// mod file_tree;

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

pub use crate::backends::Backend;
pub use file_iter::{Element, Library, Node};

pub const SUPPORTED: &'static [&'static str] = &["mp3", "ogg", "flac", "wav"];
