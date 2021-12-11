mod file_tree;

pub use anyhow::Result;
pub use hashbrown::HashSet;
pub use std::{
  path::{Path, PathBuf},
  thread,
  time::Duration,
};

pub use crate::backends::Backend;
pub use file_tree::*;
pub const SUPPORTED: &'static [&'static str] = &["mp3", "ogg", "flac", "wav"];
