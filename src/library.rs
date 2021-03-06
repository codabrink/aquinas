use crate::*;
use core::fmt;
use std::fmt::Display;

type OpenDirs = HashSet<PathBuf>;
type Dirs = HashMap<PathBuf, Arc<Node>>;
type FileList = Vec<(Arc<Node>, usize)>;

enum MaybeNode {
  Node(Arc<Node>),
  Path(PathBuf),
}

pub struct Library {
  pub root: Arc<Node>,
  pub dirs: Dirs,
  pub open_dirs: OpenDirs,
  shallow_list: FileList,
  list: FileList,
  masked_list: FileList,
  query: String,

  #[cfg(feature = "metadata")]
  _metadata: HashMap<PathBuf, Metadata>,
}

impl Library {
  pub fn new(root: impl AsRef<Path>) -> Self {
    let mut dirs = HashMap::new();
    let root = Node::new(root);
    dirs.insert(root.path.clone(), root.clone());

    let mut library = Self {
      root: root.clone(),
      dirs,
      open_dirs: HashSet::new(),
      shallow_list: Vec::new(),
      query: String::new(),
      list: Vec::new(),
      masked_list: Vec::new(),

      #[cfg(feature = "metadata")]
      _metadata: HashMap::new(),
    };

    library.open_dirs.insert(root.path.clone());
    let nodes: FileList = root.path.as_path().to_iter(&library.dirs, None).collect();
    // let mask_map = Library::build_index(root.path.clone(), &nodes);

    library.list = nodes;
    // library.mask_map = mask_map;

    library
  }

  pub fn file_list(&self) -> &FileList {
    match self.query.as_str() {
      "" => &self.shallow_list,
      _ => &self.masked_list,
    }
  }

  pub fn expand(&mut self, path: impl AsRef<Path>) {
    self.expand_all(&[path]);
  }

  pub fn expand_all(&mut self, paths: &[impl AsRef<Path>]) {
    for path in paths {
      let path = path.as_ref();
      if !path.is_dir() || self.open_dirs.get(path).is_some() {
        continue;
      }

      self.open_dirs.insert(path.to_path_buf());
    }

    self.rebuild();
  }

  pub fn collapse(&mut self, path: impl AsRef<Path>) {
    self.open_dirs.remove(path.as_ref());

    self.rebuild();
  }

  pub fn search(&mut self, query: impl AsRef<str>) {
    self.query = searchify(query.as_ref());
    self.masked_list = Vec::new();

    for (node, depth) in &self.list {
      if node.name_search.contains(&self.query) {
        self.masked_list.push((node.clone(), *depth));
      }
    }
  }

  pub fn rebuild(&mut self) {
    self.shallow_list = self
      .root
      .path
      .as_path()
      .to_iter(&self.dirs, Some(&self.open_dirs))
      // .to_iter(&self.dirs, None)
      .collect();
  }
}

pub struct DirsIter<'a> {
  dirs: &'a Dirs,
  open_dirs: Option<&'a OpenDirs>,
  stack: FileList,
  config: Config,
}

pub trait IterablePath<'a> {
  fn to_iter(&'a self, dirs: &'a Dirs, open_dirs: Option<&'a OpenDirs>) -> DirsIter<'a>;
}
impl<'a> IterablePath<'a> for &Path {
  fn to_iter(&self, dirs: &'a Dirs, open_dirs: Option<&'a OpenDirs>) -> DirsIter<'a> {
    let start_path = match self.is_dir() {
      true => self,
      false => self.parent().unwrap(),
    };

    let mut stack = vec![];
    if let Some(cursor) = dirs.get(start_path) {
      stack.push((cursor.clone(), 0));
    }

    DirsIter {
      dirs,
      open_dirs,
      stack,
      config: Config::load().unwrap_or_default(),
    }
  }
}

impl<'a> Iterator for DirsIter<'a> {
  type Item = (Arc<Node>, usize);

  fn next(&mut self) -> Option<Self::Item> {
    let (cursor, child_index) = match self.stack.last_mut() {
      Some(s) => s,
      None => return None,
    };

    if let Some(child) = cursor.child(*child_index, self.dirs) {
      *child_index += 1;
      let child = match child {
        MaybeNode::Path(p) => match self.dirs.get(&p) {
          Some(node) => node.clone(),
          _ => Node::new(p),
        },
        MaybeNode::Node(n) => n,
      };

      if self.stack.len() < self.config.scan_depth_limit {
        if self.open_dirs.is_none() || self.open_dirs.unwrap().contains(&child.path) {
          self.stack.push((child.clone(), 0));
          return Some((child, self.stack.len() - 1));
        }
      }

      return Some((child, self.stack.len()));
    }

    self.stack.pop();
    self.next()
  }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Node {
  pub path: PathBuf,
  pub files: Option<Vec<Arc<Node>>>,
  pub folders: Option<Vec<FolderKey>>,
  name: String,
  name_search: String,
  sort_key: String,

  #[cfg(feature = "metadata")]
  pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct FolderKey {
  pub path: PathBuf,
  sort_key: String,
}

impl From<PathBuf> for FolderKey {
  fn from(path: PathBuf) -> Self {
    Self {
      sort_key: path.display().to_string().to_lowercase(),
      path,
    }
  }
}

impl Node {
  pub fn is_dir(&self) -> bool {
    self.path.is_dir()
  }
  pub fn is_file(&self) -> bool {
    self.path.is_file()
  }
  pub fn title(&self) -> &str {
    #[cfg(feature = "metadata")]
    if let Some(m) = &self.metadata {
      if let Some(t) = &m.title {
        return &t;
      }
    }
    &self.name
  }

  fn child(&self, index: usize, dirs: &Dirs) -> Option<MaybeNode> {
    if let (Some(files), Some(folders)) = (&self.files, &self.folders) {
      let folders_len = folders.len();
      let files_len = files.len();

      return match index {
        i if i < folders_len => match dirs.get(&folders[i].path) {
          Some(node) => Some(MaybeNode::Node(node.clone())),
          None => Some(MaybeNode::Path(folders[i].path.clone())),
        },
        i if i < (files_len + folders_len) => Some(MaybeNode::Node(files[i - folders_len].clone())),
        _ => None,
      };
    }
    None
  }

  pub fn new(path: impl AsRef<Path>) -> Arc<Self> {
    let path = path.as_ref().to_path_buf();
    #[cfg(feature = "metadata")]
    let metadata = get_metadata(&path);
    let name = path.file_name().unwrap().to_string_lossy().to_string();

    let (files, folders) = match path.is_dir() {
      true => {
        let mut files = vec![];
        let mut folders: Vec<FolderKey> = vec![];

        if let Ok(paths) = fs::read_dir(&path) {
          for entry in paths {
            let path = entry.unwrap().path();

            if path.is_dir() {
              // better filtering in the future, for now remove the obvious junk
              if let Some(name) = path.file_name() {
                if let Some(name) = name.to_str() {
                  if name.chars().nth(0) != Some('.') {
                    folders.push(path.into());
                  }
                }
              }
            } else {
              // filter out bad files
              if let Some(ext) = path.extension() {
                if let Some(ext) = ext.to_str() {
                  if SUPPORTED.contains(&ext) {
                    files.push(Node::new(path));
                  }
                }
              }
            }
          }
        }

        files.sort_by(|a, b| a.sort_key.partial_cmp(&b.sort_key).unwrap());
        folders.sort_by(|a, b| a.sort_key.partial_cmp(&b.sort_key).unwrap());

        (Some(files), Some(folders))
      }
      false => (None, None),
    };

    Arc::new(Self {
      path,
      name_search: searchify(&name),
      sort_key: name.to_lowercase(),
      name,
      files,
      folders,

      #[cfg(feature = "metadata")]
      metadata,
    })
  }
}

fn searchify(key: &str) -> String {
  key
    .to_lowercase()
    .replace(|c: char| !c.is_alphanumeric(), "")
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

    // let mut library = Library::new(Path::new(&home).join("Music"));
    // library.rebuild();
    // let folder_count = library.root.folders.as_ref().unwrap().len();
    // let file_count = library.root.files.as_ref().unwrap().len();
    // assert_eq!(library.shallow_list.len(), folder_count + file_count);

    let library = Library::new(
      Path::new(&home)
        .join("Music")
        .join("Pendulum")
        .join("Immersion"),
    );
    assert_eq!(library.list.len(), 15);

    // println!("Mask: {:?}", library.mask_map);
  }
}
