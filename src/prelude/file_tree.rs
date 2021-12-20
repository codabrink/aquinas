use crate::{metadata::Metadata, prelude::*};
use anyhow::Result;
use std::{
  fs,
  path::{Path, PathBuf},
  rc::Rc,
};

// const STACK_SIZE: usize = 4 * 1024 * 1024;

#[derive(PartialEq)]
pub struct TreeNode {
  pub path: PathBuf,
  pub depth: usize,
  pub title: String,
  pub key: String,
  pub metadata: Option<Metadata>,
  pub children: Option<Vec<Rc<TreeNode>>>,
}

pub trait RcTreeNode {
  fn flatten(&self) -> Vec<Rc<TreeNode>>;
  fn _flatten(&self, nodes: Vec<Rc<TreeNode>>) -> Vec<Rc<TreeNode>>;
}

impl RcTreeNode for Rc<TreeNode> {
  fn flatten(&self) -> Vec<Rc<TreeNode>> {
    self._flatten(vec![])
  }
  fn _flatten(&self, mut nodes: Vec<Rc<TreeNode>>) -> Vec<Rc<TreeNode>> {
    nodes.push(self.clone());
    if let Some(children) = &self.children {
      for child in children {
        nodes = child._flatten(nodes);
      }
    }

    nodes
  }
}

pub trait AquinasPathBuf {
  fn to_tree_node(&self, expand: &HashSet<String>) -> TreeNode;
  fn as_str(&self) -> &str;
  fn file(&self) -> String;
  fn supported(&self) -> bool;
}

impl AquinasPathBuf for PathBuf {
  fn to_tree_node(&self, expand: &HashSet<String>) -> TreeNode {
    fn collect(path: &Path, depth: usize, expand: &HashSet<String>) -> Result<Vec<Rc<TreeNode>>> {
      let mut result = vec![];
      for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let is_file = path.is_file();
        let key = path.as_str().to_owned();

        if is_file && !path.supported() {
          continue;
        }

        // hide .files
        if let Some(file_name) = path.file_name() {
          if let Some(file_name) = file_name.to_str() {
            if file_name.chars().next() == Some('.') {
              continue;
            }
          }
        }

        let children = match is_file {
          false if expand.contains(&key) => Some(collect(&path, depth + 1, expand)?),
          true => None,
          false => Some(vec![]),
        };

        let metadata = crate::metadata::get_metadata(&path).ok();

        result.push(Rc::new(TreeNode {
          depth,
          key,
          title: match &metadata {
            Some(m) => match (&m.artist, &m.title) {
              (Some(a), Some(t)) => format!("{} - {}", a, t),
              _ => path.file(),
            },
            None => path.file(),
          },
          metadata,
          path,
          children,
        }));
      }

      result.sort_by(|a, b| a.key.partial_cmp(&b.key).unwrap());
      Ok(result)
    }

    TreeNode {
      depth: 0,
      path: self.clone(),
      key: self.as_str().to_owned(),
      title: self.file(),
      metadata: None,
      children: Some(collect(self, 0, expand).expect("Could not build file tree.")),
    }
  }

  fn supported(&self) -> bool {
    if let Some(ext) = self.extension() {
      if let Some(ext) = ext.to_str() {
        if SUPPORTED.contains(&&ext.to_lowercase().as_str()) {
          return true;
        }
      }
    }
    false
  }

  fn as_str(&self) -> &str {
    self.as_os_str().to_str().unwrap_or("")
  }

  fn file(&self) -> String {
    self.file_name().unwrap().to_str().unwrap().to_owned()
  }
}
