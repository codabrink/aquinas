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

    self.rebuild();
  }

  pub fn collapse(&mut self, path: impl AsRef<Path>) {
    self.open_dirs.remove(path.as_ref());

    self.rebuild();
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
      stack: vec![(cursor, 0)],
    }
  }
}

pub struct DirsIter<'a> {
  dirs: &'a OpenDirs,
  stack: Vec<(Rc<Node>, usize)>,
}

impl<'a> Iterator for DirsIter<'a> {
  type Item = (Element, usize);

  fn next(&mut self) -> Option<Self::Item> {
    let (cursor, child_index) = match self.stack.last_mut() {
      Some(s) => s,
      None => return None,
    };

    if let Some(child) = cursor.child(*child_index, &self.dirs) {
      *child_index += 1;
      match child {
        Element::Node(node) if node.is_dir() => {
          self.stack.push((node.clone(), 0));
          return Some((Element::Node(node), self.stack.len() - 1));
        }
        _ => {
          return Some((child, self.stack.len()));
        }
      }
    }

    self.stack.pop();
    self.next()
  }
}

#[derive(Clone, Debug)]
pub struct Node {
  pub path: PathBuf,
  pub metadata: Option<Metadata>,
  pub files: Option<Vec<Rc<Node>>>,
  pub folders: Option<Vec<PathBuf>>,
  name: String,
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
              // better filtering in the future, for now remove the obvious junk
              if let Some(name) = path.file_name() {
                if let Some(name) = name.to_str() {
                  if name.chars().nth(0) != Some('.') {
                    folders.push(path);
                  }
                }
              }
            } else {
              // filter out bad files
              if let Some(ext) = path.extension() {
                if let Some(ext) = ext.to_str() {
                  if SUPPORTED.contains(&ext) {
                    files.push(Rc::new(Node::new(path)));
                  }
                }
              }
            }
          }
        }

        files.sort_by(|a, b| a.path.partial_cmp(&b.path).unwrap());
        folders.sort_by(|a, b| a.partial_cmp(b).unwrap());

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
    assert_eq!(library.file_list.len(), folder_count + file_count);

    library.expand(Path::new(&home).join("Music").join("Dozer"));
    library.rebuild();
  }
}
