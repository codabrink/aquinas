use core::fmt;
use std::fmt::Display;

use crate::{
  metadata::{get_metadata, Metadata},
  prelude::*,
};

type OpenDirs = HashMap<PathBuf, Rc<Node>>;
type FileList = Vec<(Element, usize)>;

pub struct Library {
  pub root: Rc<Node>,
  pub open_dirs: OpenDirs,
  pub file_list: FileList,
}

impl Library {
  pub fn new(root: impl AsRef<Path>) -> Self {
    let root = Rc::new(Node::new(root));
    let mut open_dirs = HashMap::new();
    open_dirs.insert(root.path.clone(), root.clone());

    Self {
      root,
      open_dirs,
      file_list: vec![],
    }
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

  pub fn rebuild(&mut self) {
    self.file_list = self
      .root
      .path
      .as_path()
      .to_iter(&mut self.open_dirs)
      .collect();
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

pub struct DirsIter<'a> {
  dirs: &'a OpenDirs,
  cursor: Rc<Node>,
  child_index: usize,
  depth: usize,
}

impl<'a> Iterator for DirsIter<'a> {
  type Item = (Element, usize);

  fn next(&mut self) -> Option<Self::Item> {
    let cursor = &self.cursor;
    // Next file in the folder
    if let Some(child) = cursor.child(self.child_index, &self.dirs) {
      self.child_index += 1;
      return Some((child, self.depth));
    }
    // Go up a folder, and check for next folder / file

    if self.depth > 0 {
      self.depth -= 1;
    } else {
      return None;
    }
    match self.dirs.get(cursor.path.parent().unwrap()) {
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

#[derive(Clone, Debug)]
pub struct Node {
  pub path: PathBuf,
  pub metadata: Option<Metadata>,
  pub files: Option<Vec<Rc<Node>>>,
  pub folders: Option<Vec<PathBuf>>,
  name: String,
  expanded: bool,
}

pub enum Element {
  Node(Rc<Node>),
  Path(PathBuf, String),
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

  pub fn child(&self, index: usize, dirs: &OpenDirs) -> Option<Element> {
    if let (Some(files), Some(folders)) = (&self.files, &self.folders) {
      let folders_len = folders.len();
      let files_len = files.len();

      return match index {
        i if i < folders_len => match dirs.get(&folders[i]) {
          Some(node) => Some(Element::Node(node.clone())),
          None => {
            let path = folders[i].to_path_buf();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            Some(Element::Path(path, name))
          }
        },
        i if i < (files_len + folders_len) => Some(Element::Node(files[i - folders_len].clone())),
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
              folders.push(path);
            } else {
              files.push(Rc::new(Node::new(path)));
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
      expanded: true,
    }
  }
}

impl Display for Element {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::Node(node) => write!(f, "{}", node),
      Self::Path(_, name) => write!(f, "{}", name),
    }
  }
}

impl Element {
  pub fn path(&self) -> &Path {
    match self {
      Self::Node(node) => &node.path,
      Self::Path(path, _) => &path,
    }
  }

  pub fn is_file(&self) -> bool {
    match self {
      Self::Node(node) => node.is_file(),
      Self::Path(path, _) => path.is_file(),
    }
  }

  pub fn is_dir(&self) -> bool {
    match self {
      Self::Node(node) => node.is_dir(),
      Self::Path(path, _) => path.is_dir(),
    }
  }

  pub fn title(&self) -> &str {
    match self {
      Self::Node(node) => node.title(),
      Self::Path(_, name) => name,
    }
  }
}

impl Display for Node {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.title())
  }
}

#[cfg(test)]
mod tests {
  use std::env;
  use std::path::Path;

  use super::Library;

  #[test]
  fn exploration() {
    let home = env::var("HOME").unwrap();

    let mut library = Library::new(Path::new(&home).join("Music"));
    library.rebuild();

    let folder_count = library.root.folders.as_ref().unwrap().len();
    let file_count = library.root.files.as_ref().unwrap().len();
    // println!("len: {:?}", library.root.folders);
    assert_eq!(library.file_list.len(), folder_count + file_count);
  }
}
