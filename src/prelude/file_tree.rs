use crate::prelude::*;
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
};

#[derive(PartialEq, Clone)]
pub enum TreeNode {
    Folder(Folder),
    File(File),
}

pub struct ListElement {
    pub depth: usize,
    pub title: String,
    pub key: String,
    pub is_folder: bool,
    pub tn: Weak<TreeNode>,
}

#[derive(PartialEq, Clone)]
pub struct Folder {
    path: PathBuf,
    pub key: String,
    children: Vec<Rc<TreeNode>>,
}
#[derive(PartialEq, Clone)]
pub struct File {
    pub path: PathBuf,
    pub key: String,
}

pub trait RcTreeNode {
    fn to_list_element(&self, depth: usize) -> ListElement;
    fn key(&self) -> &String;
    fn file(&self) -> String;
    fn path(&self) -> &Path;
    fn flatten(&self, expand: &HashSet<String>) -> Vec<ListElement>;
    fn _flatten(
        &self,
        expand: &HashSet<String>,
        depth: usize,
        nodes: Vec<ListElement>,
    ) -> Vec<ListElement>;
}

impl RcTreeNode for Rc<TreeNode> {
    fn to_list_element(&self, depth: usize) -> ListElement {
        let tn = Rc::downgrade(self);
        ListElement {
            tn,
            depth,
            key: self.key().clone(),
            title: self.file(),
            is_folder: match **self {
                TreeNode::Folder(_) => true,
                _ => false,
            },
        }
    }
    fn file(&self) -> String {
        match &**self {
            TreeNode::Folder(f) => f.path.file_name(),
            TreeNode::File(f) => f.path.file_name(),
        }
        .unwrap()
        .to_string_lossy()
        .to_string()
    }
    fn path(&self) -> &Path {
        match &**self {
            TreeNode::Folder(f) => &f.path,
            TreeNode::File(f) => &f.path,
        }
    }

    fn key(&self) -> &String {
        match &**self {
            TreeNode::Folder(f) => &f.key,
            TreeNode::File(f) => &f.key,
        }
    }
    fn flatten(&self, expand: &HashSet<String>) -> Vec<ListElement> {
        self._flatten(expand, 0, vec![])
    }
    fn _flatten(
        &self,
        expand: &HashSet<String>,
        depth: usize,
        mut nodes: Vec<ListElement>,
    ) -> Vec<ListElement> {
        match &**self {
            TreeNode::Folder(folder) if expand.contains(self.key()) => {
                for child in &folder.children {
                    nodes.push(child.to_list_element(depth));
                    if let TreeNode::Folder(_) = **child {
                        nodes = child._flatten(expand, depth + 1, nodes);
                    }
                }
            }
            TreeNode::File(_) => {
                nodes.push(self.to_list_element(depth));
            }
            _ => {}
        }

        nodes
    }
}

pub trait AquinasPathBuf {
    fn to_folder(&self) -> Folder;
}

impl AquinasPathBuf for PathBuf {
    fn to_folder(&self) -> Folder {
        fn collect(path: &Path, depth: usize) -> Result<Vec<Rc<TreeNode>>> {
            let mut result = vec![];
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();

                match path.is_file() {
                    true => result.push(Rc::new(TreeNode::File(File {
                        key: path.display().to_string(),
                        path,
                    }))),
                    false => result.push(Rc::new(TreeNode::Folder(Folder {
                        children: collect(&path, depth + 1)?,
                        key: path.display().to_string(),
                        path: path,
                    }))),
                }
            }
            Ok(result)
        }

        Folder {
            path: self.clone(),
            key: self.display().to_string(),
            children: collect(self, 0).expect("Could not build file tree."),
        }
    }
}
