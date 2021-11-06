use anyhow::Result;
use std::{
    fs,
    iter::IntoIterator,
    path::{Path, PathBuf},
};

pub enum TreeNode {
    Folder(Folder),
    File(PathBuf),
}

pub struct Folder {
    depth: usize,
    expanded: bool,
    path: PathBuf,
    children: Vec<TreeNode>,
}

pub trait AquinasPathBuf {
    fn to_folder(&self) -> Folder;
}

impl AquinasPathBuf for PathBuf {
    fn to_folder(&self) -> Folder {
        fn collect(path: &Path, depth: usize) -> Result<Vec<TreeNode>> {
            let mut result = vec![];
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();

                match path.is_file() {
                    true => result.push(TreeNode::File(path)),
                    false => result.push(TreeNode::Folder(Folder {
                        depth,
                        children: collect(&path, depth + 1)?,
                        expanded: false,
                        path: path,
                    })),
                }
            }
            Ok(result)
        }

        Folder {
            expanded: true,
            depth: 0,
            path: self.clone(),
            children: collect(self, 0).expect("Could not build file tree."),
        }
    }
}
