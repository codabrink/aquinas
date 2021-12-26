use crate::{metadata::Metadata, prelude::*};
use std::rc::Rc;
use walkdir::{DirEntry, WalkDir};

enum Sort {
  File,
}

fn is_not_hidden(entry: &DirEntry) -> bool {
  entry
    .file_name()
    .to_str()
    .map(|s| entry.depth() == 0 || !s.starts_with("."))
    .unwrap_or(false)
}

#[derive(PartialEq, Default)]
pub struct TreeNode {
  pub path: PathBuf,
  pub depth: usize,
  pub file_name: String,
  pub key: String,
  pub metadata: Option<Metadata>,
  pub children: Option<Vec<Rc<TreeNode>>>,
}

fn collect(path: &Path) -> Result<Vec<TreeNode>> {
  let walker = WalkDir::new(path)
    .into_iter()
    .filter_entry(|e| is_not_hidden(e))
    .filter_map(|v| v.ok());

  for entry in walker {}

  Ok(vec![])
}

impl TreeNode {
  fn new(path: &Path, depth: usize) -> Self {
    Self {
      path: path.to_path_buf(),
      depth,
      file_name: path
        .file_name()
        .unwrap_or(OsStr::new(""))
        .to_string_lossy()
        .to_string(),
      key: path.display().to_string(),
      ..Default::default()
    }
  }
}
