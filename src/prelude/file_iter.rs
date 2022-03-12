use crate::{metadata::Metadata, prelude::*};

// A collection of expanded folders
type Dirs = HashMap<PathBuf, Dir>;

trait IterablePath<'a> {
  fn into_iter(&self, dirs: &'a mut Dirs, root: &Path) -> DirsIter<'a>;
}
impl<'a> IterablePath<'a> for Dir {
  fn into_iter(&self, dirs: &'a mut Dirs, root: &Path) -> DirsIter<'a> {
    DirsIter {
      dirs,
      cursor: self,
      child_index: 0,
    }
  }
}
impl<'a> IterablePath<'a> for File {
  fn into_iter(&self, dirs: &'a mut Dirs, root: &Path) -> DirsIter<'a> {
    let parent_path = self.path.parent().unwrap();

    let cursor = match dirs.get(parent_path) {
      Some(d) => d,
      None => {
        dirs.insert(parent_path.to_owned(), Dir::new(parent_path, 0));
        dirs.get(parent_path).unwrap()
      }
    };

    let child_index = cursor
      .children
      .iter()
      .position(|c| {
        if let Child::File(f) = c {
          return f == self;
        }
        false
      })
      .unwrap_or(0);

    DirsIter {
      dirs,
      cursor,
      child_index,
    }
  }
}

struct DirsIter<'a> {
  dirs: &'a Dirs,
  cursor: *const Dir,
  child_index: usize,
}

impl<'a> DirsIter<'a> {
  fn cursor(&self) -> &'a Dir {
    unsafe { &*(self.cursor) }
  }
}

impl<'a> Iterator for DirsIter<'a> {
  type Item = &'a Child;

  fn next(&mut self) -> Option<Self::Item> {
    self.child_index += 1;

    if let Some(child) = self.cursor().children.get(self.child_index) {
      return Some(child);
    }
    None
  }
}

struct Dir {
  path: PathBuf,
  children: Vec<Child>,
  depth: usize,
}

#[derive(PartialEq)]
struct File {
  path: PathBuf,
  metadata: Option<Metadata>,
}

enum Child {
  File(File),
  Dir(PathBuf),
}

impl Dir {
  fn new(path: impl AsRef<Path>, depth: usize) -> Self {
    let path = path.as_ref().to_path_buf();
    let mut children = vec![];

    if let Ok(paths) = fs::read_dir(&path) {
      for path in paths {
        let path = path.unwrap().path();

        if path.is_file() {
          children.push(Child::File(File {
            path,
            metadata: None,
          }))
        } else {
          children.push(Child::Dir(path));
        }
      }
    }

    Self {
      path,
      children,
      depth,
    }
  }
}
