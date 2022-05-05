mod file_list;
mod player_state;
mod user_input;

use crate::*;
use anyhow::Result;
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
  boxed::Box,
  io,
  path::Path,
  time::{Duration, Instant},
};
use tui::{
  backend::{Backend, CrosstermBackend},
  layout::{Constraint, Direction, Layout},
  widgets::ListState,
  Terminal,
};

#[derive(PartialEq)]
pub enum Focusable {
  FileList,
  Dir,
  Search,
}

pub struct App {
  pub backend: Box<dyn AudioBackend>,
  pub library: Library,
  pub list_index: usize,
  pub list_offset: usize,
  pub height: i32,
  pub input: String,
  pub focus: Focusable,
  pub progress: (f64, u64, u64),
  pub playing: Option<Arc<Node>>,
  pub play_index: usize,
}

impl App {
  pub fn new() -> Self {
    let path = std::env::current_dir().expect("Could not get current dir.");
    let backend = backends::load();

    let progress = backend.progress();

    let mut app = Self {
      backend,
      library: Library::new(&path),
      playing: None,
      height: 0,
      list_index: 0,
      list_offset: 0,
      focus: Focusable::FileList,
      input: String::new(),
      progress,
      play_index: 0,
    };
    // app.set_root(&path);

    // Development code
    app.focus = Focusable::Dir;
    app.input = "~/Music".to_owned();
    user_input::process_cmd(&mut app);

    app.library.rebuild();

    app
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

  pub fn run_app(&mut self) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
      self.draw(&mut terminal, &mut list_state)?;

      let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));

      if crossterm::event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
          self.on_key(&key, &mut terminal, &mut list_state)?;
        }
      }

      if last_tick.elapsed() > tick_rate {
        self.on_tick()?;
        last_tick = Instant::now();
      }
    }
  }

  fn on_key<B: Backend + io::Write>(
    &mut self,
    key: &KeyEvent,
    terminal: &mut Terminal<B>,
    list_state: &mut ListState,
  ) -> Result<()> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match (key.code, ctrl) {
      (KeyCode::Char('q'), _) => {
        disable_raw_mode()?;
        execute!(
          terminal.backend_mut(),
          LeaveAlternateScreen,
          DisableMouseCapture
        )?;
        std::process::exit(0);
      }
      (KeyCode::Down, _) | (KeyCode::Char('n'), true) => {
        self.list_index =
          (self.list_index + 1).min(self.library.file_list().len().saturating_sub(1));
        self.list_offset = self.list_offset.max(
          self
            .list_index
            .saturating_sub(self.height.saturating_sub(3) as usize),
        );
        list_state.select(Some(self.list_index.saturating_sub(self.list_offset)));
      }
      (KeyCode::Up, _) | (KeyCode::Char('p'), true) => {
        self.list_index = self.list_index.saturating_sub(1);
        self.list_offset = self.list_offset.min(self.list_index);
        list_state.select(Some(self.list_index.saturating_sub(self.list_offset)));
      }
      _ => match self.focus {
        Focusable::FileList => file_list::handle_input(self, list_state, key),
        Focusable::Dir | Focusable::Search => user_input::handle_input(self, key),
      },
    }

    Ok(())
  }

  fn draw<B: Backend>(
    &mut self,
    terminal: &mut Terminal<B>,
    list_state: &mut ListState,
  ) -> Result<()> {
    terminal.draw(|f| {
      self.height = f.size().height as i32;

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

      file_list::render_file_list(self, chunks[chunks.len() - 2], f, list_state);
      player_state::render(self, &chunks.last().unwrap(), f);
    })?;

    Ok(())
  }

  fn on_tick(&mut self) -> Result<()> {
    self.progress = self.backend.progress();

    self.ensure_continue();

    Ok(())
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