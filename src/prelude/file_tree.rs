use anyhow::Result;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

pub enum TreeNode {
    Folder(Folder),
    File(File),
}
pub enum BorrowedTreeNode<'a> {
    Folder(&'a Folder),
    File(&'a File),
}

impl<'a> fmt::Display for BorrowedTreeNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowedTreeNode::Folder(folder) => {
                write!(
                    f,
                    "{}{}",
                    " ".repeat(folder.depth),
                    folder.path.file_name().unwrap().to_string_lossy()
                )
            }
            BorrowedTreeNode::File(file) => {
                write!(
                    f,
                    "{}{}",
                    " ".repeat(file.depth),
                    file.path.file_name().unwrap().to_string_lossy()
                )
            }
        }
    }
}

impl<'a> From<&BorrowedTreeNode<'a>> for String {
    fn from(tn: &BorrowedTreeNode<'a>) -> Self {
        format!("{}", tn)
    }
}

pub struct Folder {
    depth: usize,
    expanded: bool,
    path: PathBuf,
    children: Vec<TreeNode>,
}
pub struct File {
    depth: usize,
    path: PathBuf,
}

impl Folder {
    pub fn flatten(&'a self) -> Vec<BorrowedTreeNode> {
        fn flatten(folder: &'a Folder, result: &mut Vec<BorrowedTreeNode<'a>>) {
            result.push(BorrowedTreeNode::Folder(folder));

            for child in &folder.children {
                match &child {
                    TreeNode::File(f) => result.push(BorrowedTreeNode::File(f)),
                    TreeNode::Folder(f) if f.expanded => flatten(f, result),
                    TreeNode::Folder(f) => result.push(BorrowedTreeNode::Folder(f)),
                }
            }
        }

        let mut result = vec![];
        flatten(self, &mut result);
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
