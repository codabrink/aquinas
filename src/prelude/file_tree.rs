use crate::prelude::*;
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

        if is_file {
          match path.extension() {
            Some(ext) => match ext.to_str() {
              Some(ext) => {
                if !SUPPORTED.contains(&&ext.to_lowercase().as_str()) {
                  continue;
                }
              }
              _ => continue,
            },
            _ => continue,
          }
        }

        let children = match is_file {
          false if expand.contains(&key) => Some(collect(&path, depth + 1, expand)?),
          true => None,
          false => Some(vec![]),
        };

        result.push(Rc::new(TreeNode {
          depth,
          key,
          title: path.file(),
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
      children: Some(collect(self, 0, expand).expect("Could not build file tree.")),
    }
  }

  fn as_str(&self) -> &str {
    self.as_os_str().to_str().unwrap_or("")
  }

  fn file(&self) -> String {
    self.file_name().unwrap().to_str().unwrap().to_owned()
  }
}
