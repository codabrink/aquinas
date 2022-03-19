use core::fmt;
use std::fmt::Display;

use crate::{
  metadata::{get_metadata, Metadata},
  prelude::*,
};

type OpenDirs = HashMap<PathBuf, Rc<Node>>;
type FileList = Vec<(Rc<Node>, usize)>;

pub struct Library {
  pub root: PathBuf,
  pub open_dirs: OpenDirs,
  pub file_list: FileList,
}

impl Library {
  pub fn new(root: impl AsRef<Path>) -> Self {
    let mut library = Self {
      root: root.as_ref().to_path_buf(),
      open_dirs: HashMap::new(),
      file_list: vec![],
    };
    library.expand(root);
    library
  }

  pub fn expand(&mut self, path: impl AsRef<Path>) {
    let path = path.as_ref();
    if !path.is_dir() || self.open_dirs.get(path).is_some() {
      return;
    }

    self
      .open_dirs
      .insert(path.to_path_buf(), Rc::new(Node::new(path)));
  }

  pub fn collapse(&mut self, path: impl AsRef<Path>) {
    self.open_dirs.remove(path.as_ref());
  }

  fn rebuild(&mut self) {
    self.file_list = self.root.as_path().to_iter(&mut self.open_dirs).collect();
  }
}

pub trait IterablePath<'a> {
  fn to_iter(&'a self, dirs: &'a OpenDirs) -> DirsIter<'a>;
}
impl<'a> IterablePath<'a> for &Path {
  fn to_iter(&self, dirs: &'a OpenDirs) -> DirsIter<'a> {
    let start_path = match self.is_dir() {
      true => self,
      false => self.parent().unwrap(),
    };

    let cursor = match dirs.get(start_path) {
      Some(d) => d,
      None => panic!("Bug: open dirs should contain the iterating path."),
    }
    .clone();

    DirsIter {
      dirs,
      cursor,
      child_index: 0,
      depth: 0,
    }
  }
}

struct DirsIter<'a> {
  dirs: &'a OpenDirs,
  cursor: Rc<Node>,
  child_index: usize,
  depth: usize,
}

impl<'a> Iterator for DirsIter<'a> {
  type Item = (Rc<Node>, usize);

  fn next(&mut self) -> Option<Self::Item> {
    let cursor = &self.cursor;
    // Next file in the folder
    if let Some(child) = cursor.child(self.child_index, &self.dirs) {
      self.child_index += 1;
      return Some((child, self.depth));
    }
    // Go up a folder, and check for next folder / file
    let parent_path = cursor.path.parent().unwrap();
    self.depth -= 1;
    match self.dirs.get(parent_path) {
      None => {
        // If none, this means we're past the root, and there is nothing else to iter through
        return None;
      }
      // Recurse
      Some(dir) => {
        self.cursor = dir.clone();
        self.child_index = 0;
        self.depth += 1;
        return self.next();
      }
    }
  }
}

#[derive(Clone)]
pub struct Node {
  pub path: PathBuf,
  pub metadata: Option<Metadata>,
  pub files: Option<Vec<Rc<Node>>>,
  pub folders: Option<Vec<PathBuf>>,
  name: String,
}
pub enum Child<'a> {
  File(Rc<Node>),
  Folder(&'a Path),
}

impl Node {
  pub fn is_dir(&self) -> bool {
    self.path.is_dir()
  }
  pub fn is_file(&self) -> bool {
    self.path.is_file()
  }
  pub fn title(&self) -> &str {
    if let Some(m) = &self.metadata {
      if let Some(t) = &m.title {
        return &t;
      }
    }
    &self.name
  }

  pub fn child(&self, index: usize, dirs: &OpenDirs) -> Option<Rc<Node>> {
    if let (Some(files), Some(folders)) = (&self.files, &self.folders) {
      let folders_len = folders.len();
      let files_len = files.len();

      return match index {
        i if i < folders_len => Some(dirs.get(&folders[i]).unwrap().clone()),
        i if i < (files_len + folders_len) => Some(files[i - folders_len].clone()),
        _ => None,
      };
    }
    None
  }

  pub fn new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref().to_path_buf();
    let metadata = get_metadata(&path);
    let name = path.file_name().unwrap().to_string_lossy().to_string();

    let (files, folders) = match path.is_dir() {
      true => {
        let mut files = vec![];
        let mut folders = vec![];
        if let Ok(paths) = fs::read_dir(&path) {
          for entry in paths {
            let path = entry.unwrap().path();

            if path.is_dir() {
              files.push(Rc::new(Node::new(path)));
            } else {
              folders.push(path);
            }
          }
        }
        (Some(files), Some(folders))
      }
      false => (None, None),
    };

    Self {
      path,
      metadata,
      name,
      files,
      folders,
    }
  }
}

impl Display for Node {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.title())
  }
}
