use crate::prelude::*;
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(PartialEq)]
pub enum TreeNode {
    Folder(Folder),
    File(File),
}

#[derive(PartialEq)]
pub enum BorrowedTreeNode<'a> {
    Folder(&'a Folder),
    File(&'a File),
}

impl<'a> PartialOrd for BorrowedTreeNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path_string().partial_cmp(other.path_string())
    }
    fn lt(&self, other: &Self) -> bool {
        self.path_string() < other.path_string()
    }
    fn le(&self, other: &Self) -> bool {
        self.path_string() <= other.path_string()
    }
    fn gt(&self, other: &Self) -> bool {
        self.path_string() > other.path_string()
    }
    fn ge(&self, other: &Self) -> bool {
        self.path_string() >= other.path_string()
    }
}

impl<'a> BorrowedTreeNode<'a> {
    pub fn file(&self) -> String {
        match self {
            BorrowedTreeNode::Folder(f) => f.path.file_name(),
            BorrowedTreeNode::File(f) => f.path.file_name(),
        }
        .unwrap()
        .to_string_lossy()
        .to_string()
    }
    fn path_string(&self) -> &String {
        match self {
            BorrowedTreeNode::Folder(f) => &f.path_string,
            BorrowedTreeNode::File(f) => &f.path_string,
        }
    }
}

#[derive(PartialEq)]
pub struct Folder {
    pub depth: usize,
    path: PathBuf,
    pub path_string: String,
    children: Vec<TreeNode>,
}
#[derive(PartialEq)]
pub struct File {
    pub depth: usize,
    pub path: PathBuf,
    pub path_string: String,
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
                    TreeNode::Folder(f) if expanded.contains(&f.path_string) => {
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
                    true => result.push(TreeNode::File(File {
                        depth,
                        path_string: path.display().to_string(),
                        path,
                    })),
                    false => result.push(TreeNode::Folder(Folder {
                        depth,
                        children: collect(&path, depth + 1)?,
                        path_string: path.display().to_string(),
                        path: path,
                    })),
                }
            }
            Ok(result)
        }

        Folder {
            depth: 0,
            path: self.clone(),
            path_string: self.display().to_string(),
            children: collect(self, 0).expect("Could not build file tree."),
        }
    }
}
