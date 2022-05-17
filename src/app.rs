mod file_list;
mod player_state;
mod user_input;
#[cfg(feature = "mpris")]
use crate::mpris::{build_metadata, PlaybackStatus};
use crate::*;
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
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

pub enum AppMessage {
  Select(usize),
  SelectDelta(i64),
  Play(Option<PathBuf>),
  Pause,
  PlayPause,
  Next,
  Prev,
}

pub struct App {
  pub backend: Box<dyn AudioBackend>,
  pub library: Library,
  pub height: i32,
  pub input: String,
  pub focus: Focusable,
  pub selected: Option<usize>,
  pub window_offset: usize,
  pub progress: (f64, u64, u64),
  pub playing: Option<Arc<Node>>,
  pub play_index: usize,
  mailbox: (Arc<Sender<AppMessage>>, Receiver<AppMessage>),
  last_played: Option<Arc<Node>>,

  #[cfg(feature = "mpris")]
  mpris_tx: Sender<PlaybackStatus>,
}

impl App {
  pub fn new() -> Self {
    let path = std::env::current_dir().expect("Could not get current dir.");
    let backend = backends::load();

    let progress = backend.progress();

    let (sender, receiver) = crossbeam_channel::unbounded();
    let sender = Arc::new(sender);

    #[cfg(feature = "mpris")]
    let mpris_tx = {
      let (mpris_tx, mpris_rx) = crossbeam_channel::unbounded();
      let mailbox = sender.clone();

      std::thread::spawn(move || {
        let _ = mpris::run_dbus_server(mailbox, mpris_rx);
      });

      mpris_tx
    };

    let mut app = Self {
      backend,
      library: Library::new(&path),
      playing: None,
      height: 0,
      focus: Focusable::FileList,
      input: String::new(),
      selected: None,
      window_offset: 0,
      progress,
      play_index: 0,
      mailbox: (sender, receiver),
      last_played: None,

      #[cfg(feature = "mpris")]
      mpris_tx,
    };
    // app.set_root(&path);

    if let Ok(meta) = Meta::load() {
      if let Some(last_path) = meta.last_path {
        app.focus = Focusable::Dir;
        app.input = last_path.display().to_string();
        user_input::process_cmd(&mut app);
      }
    }

    app.library.rebuild();

    app
  }

  pub fn message(&self, msg: AppMessage) {
    let _ = self.mailbox.0.send(msg);
  }

  pub fn set_root(&mut self, path: &Path) {
    let mut library = Library::new(path);
    for path in &self.library.open_dirs {
      if path.starts_with(&library.root.path) {
        library.expand(path);
      }
    }
    library.rebuild();
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
      let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));

      if last_tick.elapsed() > tick_rate {
        last_tick = Instant::now();
      }

      if crossterm::event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
          self.on_key(&key, &mut terminal)?;
        }
      }

      self.update(&mut list_state)?;
      self.draw(&mut terminal, &mut list_state)?;
    }
  }

  fn on_key<B: Backend + io::Write>(
    &mut self,
    key: &KeyEvent,
    terminal: &mut Terminal<B>,
  ) -> Result<()> {
    use AppMessage::*;

    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    match (key.code, ctrl, alt) {
      (KeyCode::Char('q'), _, _) => {
        disable_raw_mode()?;
        execute!(
          terminal.backend_mut(),
          LeaveAlternateScreen,
          DisableMouseCapture
        )?;
        std::process::exit(0);
      }
      (KeyCode::Down, _, _) | (KeyCode::Char('n'), true, _) => {
        self.message(SelectDelta(1));
      }
      (KeyCode::Up, _, _) | (KeyCode::Char('p'), true, _) => {
        self.message(SelectDelta(-1));
      }
      (KeyCode::Char('v'), true, _) => {
        self.message(SelectDelta(10));
      }
      (KeyCode::Char('v'), _, true) => {
        self.message(SelectDelta(-10));
      }
      _ => match self.focus {
        Focusable::FileList => file_list::handle_input(self, key),
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
      self.height = f.size().height as i32 - 1;

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

  fn update(&mut self, list_state: &mut ListState) -> Result<()> {
    use AppMessage::*;

    self.progress = self.backend.progress();

    self.ensure_continue();

    let messages: Vec<AppMessage> = self.mailbox.1.try_iter().collect();
    for msg in messages {
      match msg {
        Select(index) => {
          self.select(index, list_state);
        }
        SelectDelta(delta) => {
          let index = self.selected.unwrap_or(0) as i64 + delta;
          self.select(index.max(0) as usize, list_state);
        }
        Play(_path) => self.backend.play(None)?,
        PlayPause => self.backend.play_pause(),
        Pause => self.backend.pause(),
        Next => self.play(self.play_index + 1),
        Prev => self.play(self.play_index.saturating_sub(1)),
      }
    }

    Ok(())
  }

  pub fn highlighted(&mut self) -> Option<Arc<Node>> {
    if let Some(selected) = self.selected {
      if let Some((node, _)) = self.library.file_list().get(selected) {
        return Some(node.clone());
      }
    }
    None
  }

  pub fn play(&mut self, index: usize) {
    self.play_index = index;
    match self.library.file_list().get(index) {
      Some((node, _)) => {
        if node.is_file() {
          #[cfg(feature = "mpris")]
          {
            let _ = self
              .mpris_tx
              .send(PlaybackStatus::Playing(Some(build_metadata(node))));
          }

          self.last_played = Some(node.clone());
          self.backend.play(Some(&node.path));
          return;
        }

        self.expand(index);
        self.play(index + 1);
      }
      None => {
        self.pause();
      }
    }
    self.message(AppMessage::Select(index));
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

  pub fn pause(&mut self) {
    self.backend.pause();

    #[cfg(feature = "mpris")]
    let _ = self.mpris_tx.send(PlaybackStatus::Paused);
  }

  pub fn play_pause(&mut self) {
    self.backend.play_pause();

    #[cfg(feature = "mpris")]
    let _ = match self.backend.is_paused() {
      true => self.mpris_tx.send(PlaybackStatus::Paused),
      false => self.mpris_tx.send(PlaybackStatus::Playing(
        self.last_played.as_ref().map(|lp| build_metadata(&lp)),
      )),
    };
  }

  fn select(&mut self, index: usize, list_state: &mut ListState) {
    let height = self.height as usize - 1;
    let index = index.min(self.library.file_list().len());

    self.selected = Some(index);
    self.window_offset = self
      .window_offset
      .max(index.saturating_sub(height))
      .min(index);
    list_state.select(Some(index - self.window_offset));
  }

  pub fn expand(&mut self, index: usize) {
    if let Some((node, _)) = self.library.file_list().get(index) {
      let path = node.path.clone();
      self.library.expand(path);
    }
  }

  pub fn view_range(&self) -> Range<usize> {
    self.window_offset..(self.window_offset + self.height as usize)
  }

  pub fn collapse(&mut self, index: usize) {
    if let Some((node, _)) = self.library.file_list().get(index) {
      let path = node.path.clone();
      self.library.collapse(path);
    }
  }

  #[inline]
  fn ensure_continue(&mut self) {
    if self.backend.track_finished() {
      self.play(self.play_index + 1);
    }
  }
}
