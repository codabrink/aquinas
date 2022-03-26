mod file_list;
mod player_state;
mod user_input;

use crate::*;
use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver};
use std::{
  io::{self, Stdout},
  path::Path,
  thread,
  time::Duration,
};
use termion::{
  event::Key,
  input::TermRead,
  raw::{IntoRawMode, RawTerminal},
  screen::AlternateScreen,
};
use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout},
  widgets::ListState,
  Terminal as TuiTerminal,
};

pub type Frame<'a> = tui::Frame<'a, CrosstermBackend<AlternateScreen<RawTerminal<Stdout>>>>;

enum Event {
  Input(Key),
  Tick,
}

#[derive(PartialEq)]
pub enum Focusable {
  FileList,
  Dir,
  Search,
}

pub struct Interface {
  evt_rx: Receiver<Event>,
  pub backend: Box<dyn Backend>,
  pub library: Library,
  pub list_index: usize,
  pub list_offset: usize,
  pub input: String,
  pub focus: Focusable,
  pub progress: (f64, u64, u64),
  pub playing: Option<Arc<Node>>,
  pub play_index: usize,
}

impl Interface {
  pub fn new() -> Self {
    let (evt_tx, evt_rx) = unbounded();

    // stdin read loop
    thread::spawn({
      let evt_tx = evt_tx.clone();
      move || {
        let stdin = io::stdin();
        for evt in stdin.keys() {
          if let Ok(key) = evt {
            let _ = evt_tx.send(Event::Input(key));
          }
        }
      }
    });

    // tick loop
    thread::spawn(move || loop {
      let _ = evt_tx.send(Event::Tick);
      thread::sleep(Duration::from_millis(1000));
    });

    let path = std::env::current_dir().expect("Could not get current dir.");
    let backend = backends::load();
    let progress = backend.progress();

    let mut interface = Self {
      backend,
      evt_rx,
      library: Library::new(&path),
      playing: None,
      list_index: 0,
      list_offset: 0,
      focus: Focusable::FileList,
      input: String::new(),
      progress,
      play_index: 0,
    };
    interface.set_root(&path);

    // Development code
    interface.focus = Focusable::Dir;
    interface.input = "~/Music".to_owned();
    user_input::process_cmd(&mut interface);

    interface.library.rebuild();

    interface
  }

  pub fn set_root(&mut self, path: &Path) {
    let mut library = Library::new(path);
    for path in &self.library.open_dirs {
      if path.starts_with(&library.root.path) {
        library.expand(path);
      }
    }
    self.library = library;
  }

  pub fn render_loop(&mut self) -> Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = TuiTerminal::new(backend)?;

    let mut list_state = ListState::default();
    let mut height = 0;

    loop {
      self.progress = self.backend.progress();

      terminal.draw(|f| {
        height = f.size().height;

        let v_constraints = match self.focus {
          Focusable::Dir | Focusable::Search => {
            vec![
              Constraint::Length(3),
              Constraint::Min(1),
              Constraint::Length(2),
            ]
          }
          _ => vec![Constraint::Min(1), Constraint::Length(2)],
        };

        let chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints(v_constraints)
          .split(f.size());

        match self.focus {
          Focusable::Dir | Focusable::Search => {
            user_input::render(self, chunks[0], f);
          }
          _ => {}
        }

        file_list::render_file_list(self, &mut list_state, chunks[chunks.len() - 2], f);
        player_state::render(self, &chunks.last().unwrap(), f);
      })?;

      self.ensure_continue();

      match self.evt_rx.recv()? {
        Event::Input(key) => match key {
          Key::Ctrl('q') | Key::Esc => {
            drop(terminal);
            std::process::exit(0);
          }
          Key::Down | Key::Ctrl('n') => {
            self.list_index =
              (self.list_index + 1).min(self.library.file_list().len().saturating_sub(1));
            self.list_offset = self.list_offset.max(
              self
                .list_index
                .saturating_sub(height.saturating_sub(3) as usize),
            );
            list_state.select(Some(self.list_index.saturating_sub(self.list_offset)));
          }
          Key::Up | Key::Ctrl('p') => {
            self.list_index = self.list_index.saturating_sub(1);
            self.list_offset = self.list_offset.min(self.list_index);
            list_state.select(Some(self.list_index.saturating_sub(self.list_offset)));
          }
          _ => match self.focus {
            Focusable::FileList => file_list::handle_input(self, &mut list_state, key),
            Focusable::Dir | Focusable::Search => user_input::handle_input(self, key),
          },
        },
        _ => {}
      }
    }
  }

  pub fn highlighted(&mut self) -> Option<Arc<Node>> {
    self
      .library
      .file_list()
      .get(self.list_index + self.list_offset)
      .map(|(n, _)| n.clone())
  }

  pub fn play(&mut self, index: usize) {
    self.play_index = index;
    match self.library.file_list().get(index) {
      Some((node, _)) => {
        if node.is_file() {
          self.backend.play(&node.path);
          return;
        }
        self.expand(index);
        self.play(index + 1);
      }
      None => {
        self.backend.pause();
      }
    }
  }

  pub fn play_path(&mut self, path: impl AsRef<Path>) {
    let path = path.as_ref();
    if !path.starts_with(&self.library.root.path) {
      // current root does not contain path
      return;
    }

    // expand the path and it's parents
    let mut bubble = path;
    let mut bubble_paths = vec![path];

    while bubble != self.library.root.path {
      bubble_paths.push(bubble);
      bubble = match bubble.parent() {
        Some(p) => p,
        _ => break,
      };
    }
    self.library.expand_all(&bubble_paths);

    let index = self
      .library
      .file_list()
      .iter()
      .position(|(n, _)| n.path == path);
    if let Some(index) = index {
      self.play(index);
    }
  }

  pub fn expand(&mut self, index: usize) {
    if let Some((node, _)) = self.library.file_list().get(index) {
      let path = node.path.clone();
      self.library.expand(path);
    }
  }

  pub fn collapse(&mut self, index: usize) {
    if let Some((node, _)) = self.library.file_list().get(index) {
      let path = node.path.clone();
      self.library.collapse(path);
    }
  }

  fn ensure_continue(&mut self) {
    if self.progress.2 == 0 || self.progress.1 != self.progress.2 || self.backend.is_paused() {
      return;
    }

    self.play(self.play_index + 1);
  }
}
