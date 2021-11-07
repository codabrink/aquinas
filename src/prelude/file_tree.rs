use crate::prelude::*;
use anyhow::Result;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};
use tui::{text::Span, widgets::ListItem};

pub enum TreeNode {
    Folder(Folder),
    File(File),
}
pub enum BorrowedTreeNode<'a> {
    Folder(&'a Folder),
    File(&'a File),
}

impl<'a> BorrowedTreeNode<'a> {
    fn file(&self) -> String {
        match self {
            BorrowedTreeNode::Folder(f) => f.path.file_name(),
            BorrowedTreeNode::File(f) => f.path.file_name(),
        }
        .unwrap()
        .to_string_lossy()
        .to_string()
    }

    pub fn to_string(&self, expanded: &HashSet<String>) -> String {
        match self {
            BorrowedTreeNode::File(f) => {
                format!("{}{}", " ".repeat(f.depth * 2), self.file())
            }
            BorrowedTreeNode::Folder(f) => {
                let symbol = if expanded.contains(&f.display) {
                    '▼'
                } else {
                    '▶'
                };
                format!("{}{} {}", " ".repeat(f.depth * 2), symbol, self.file())
            }
        }
    }
}

pub struct Folder {
    depth: usize,
    path: PathBuf,
    pub display: String,
    children: Vec<TreeNode>,
}
pub struct File {
    depth: usize,
    pub path: PathBuf,
}

impl Folder {
    pub fn flatten(&'a self, expanded: &HashSet<String>) -> Vec<BorrowedTreeNode> {
        fn flatten(
            folder: &'a Folder,
            expanded: &HashSet<String>,
            result: &mut Vec<BorrowedTreeNode<'a>>,
        ) {
            result.push(BorrowedTreeNode::Folder(folder));

            for child in &folder.children {
                match &child {
                    TreeNode::File(f) => result.push(BorrowedTreeNode::File(f)),
                    TreeNode::Folder(f) if expanded.contains(&f.display) => {
                        flatten(f, expanded, result)
                    }
                    TreeNode::Folder(f) => result.push(BorrowedTreeNode::Folder(f)),
                }
            }
        }

        let mut result = vec![];
        flatten(self, expanded, &mut result);
        result
    }
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
                    true => result.push(TreeNode::File(File { path, depth })),
                    false => result.push(TreeNode::Folder(Folder {
                        depth,
                        children: collect(&path, depth + 1)?,
                        display: format!("{}", path.display()),
                        path: path,
                    })),
                }
            }
            Ok(result)
        }

        Folder {
            depth: 0,
            path: self.clone(),
            display: format!("{}", self.display()),
            children: collect(self, 0).expect("Could not build file tree."),
        }
    }
}
