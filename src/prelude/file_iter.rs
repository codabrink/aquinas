use crate::{
  metadata::{get_metadata, Metadata},
  prelude::*,
};

// A collection of expanded folders
pub type Dirs = HashMap<PathBuf, Dir>;

pub trait DirsTrait<'a> {
  fn expand(&mut self, path: impl AsRef<Path>) -> bool;
  fn collapse(&mut self, path: impl AsRef<Path>);
  fn collect(&'a self, root: impl AsRef<Path>) -> Vec<(BNode<'a>, usize)>;
}

impl<'a> DirsTrait<'a> for Dirs {
  fn expand(&mut self, path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if !path.is_dir() || self.get(path).is_some() {
      return false;
    }

    let dir = Dir::new(path);
    self.insert(path.to_path_buf(), dir);
    true
  }

  fn collapse(&mut self, path: impl AsRef<Path>) {
    self.remove(path.as_ref());
  }

  fn collect(&'a self, root: impl AsRef<Path>) -> Vec<(BNode<'a>, usize)> {
    let result = vec![];
    let mut depth = 0;
    let root = match self.get(root.as_ref()) {
      Some(dir) => dir,
      _ => return vec![],
    };

    // Stack: Vec<(dir, index)>
    let mut stack = vec![(root, 0)];
    loop {
      let (dir, index) = match stack.last() {
        Some(s) => s,
        None => {
          return result;
        }
      };

      match dir.children.get(*index) {
        Some(Node::Dir((p, _))) => {
          result.push((BNode::Dir(p), depth));

          if let Some(child_dir) = self.get(p) {
            // folder is expanded, push to the stack
            stack.push((child_dir, 0));
            depth += 1;
          }
        }
        Some(Node::File(file)) => {
          result.push((BNode::File(file), depth));
        }
        // Nothing here, pop up the stack
        None => {
          stack.pop();
          depth -= 1;
        }
      }

      // next
      if let Some((_, i)) = stack.last_mut() {
        *i += 1;
      }
    }
  }
}

pub trait IterablePath<'a> {
  fn iter(&self, dirs: &'a mut Dirs, root: &Path) -> DirsIter<'a>;
}
impl<'a> IterablePath<'a> for Dir {
  fn iter(&self, dirs: &'a mut Dirs, root: &Path) -> DirsIter<'a> {
    let cursor = match dirs.get(&self.path) {
      Some(d) => d,
      None => {
        dirs.insert(self.path.clone(), self.clone());
        dirs.get(&self.path).unwrap()
      }
    };

    DirsIter {
      dirs,
      cursor,
      child_index: 0,
      depth: 0,
    }
  }
}
impl<'a> IterablePath<'a> for File {
  fn iter(&self, dirs: &'a mut Dirs, root: &Path) -> DirsIter<'a> {
    let parent_path = self.path.parent().unwrap();

    let cursor = match dirs.get(parent_path) {
      Some(d) => d,
      None => {
        dirs.insert(parent_path.to_owned(), Dir::new(parent_path));
        dirs.get(parent_path).unwrap()
      }
    };

    let child_index = cursor
      .children
      .iter()
      .position(|c| {
        if let Node::File(f) = c {
          return f == self;
        }
        false
      })
      .unwrap_or(0);

    DirsIter {
      dirs,
      cursor,
      child_index,
      depth: 0,
    }
  }
}

struct DirsIter<'a> {
  dirs: &'a Dirs,
  cursor: *const Dir,
  child_index: usize,
  depth: usize,
}

impl<'a> DirsIter<'a> {
  fn cursor(&self) -> &'a Dir {
    unsafe { &*(self.cursor) }
  }
}

impl<'a> Iterator for DirsIter<'a> {
  type Item = (&'a Node, usize);

  fn next(&mut self) -> Option<Self::Item> {
    let cursor = self.cursor();
    // Next file in the folder
    if let Some(child) = cursor.children.get(self.child_index) {
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
        self.cursor = dir;
        self.child_index = 0;
        self.depth += 1;
        return self.next();
      }
    }
  }
}

#[derive(Clone)]
pub struct Dir {
  pub path: PathBuf,
  pub children: Vec<Node>,
  pub name: String,
}

#[derive(PartialEq, Clone)]
pub struct File {
  pub path: PathBuf,
  pub metadata: Option<Metadata>,
  pub name: String,
}

#[derive(Clone)]
pub enum Node {
  File(File),
  Dir((PathBuf, String)),
}

impl Node {
  pub fn is_dir(&self) -> bool {
    if let Self::Dir(_) = self {
      return true;
    }
    false
  }
  pub fn is_file(&self) -> bool {
    if let Self::File(_) = self {
      return true;
    }
    false
  }
  pub fn title(&self) -> &str {
    match self {
      Self::Dir((_, name)) => name,
      Self::File(f) => f.name,
    }
  }
}

pub enum BNode<'a> {
  File(&'a File),
  Dir(&'a Path),
}

impl File {
  pub fn new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref().to_path_buf();
    let metadata = get_metadata(&path);
    let file_name = path.file_name().to_string_lossy().to_string();
    Self {
      name: metadata.map(|m| m.title).unwrap_or(file_name),
      metadata,
      path,
    }
  }
}

impl Dir {
  pub fn new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref().to_path_buf();
    let name = path
      .file_name()
      .unwrap_or(OsStr::new(""))
      .to_string_lossy()
      .to_string();
    let mut children = vec![];

    if let Ok(paths) = fs::read_dir(&path) {
      for path in paths {
        let path = path.unwrap().path();

        if path.is_file() {
          children.push(Node::File(File::new(&path)));
        } else {
          children.push(Node::Dir((path, name.clone())));
        }
      }
    }

    Self {
      path,
      children,
      name,
    }
  }
}
