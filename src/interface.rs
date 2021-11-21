mod file_list;
mod player_state;
mod user_input;

use crate::{backends, prelude::*};
use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver};
use std::{
    io::{self, Stdout},
    path::{Path, PathBuf},
    rc::Rc,
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
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
    Terminal as TuiTerminal,
};

pub type Frame<'a> = tui::Frame<'a, TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>;

enum Event {
    Input(Key),
    Tick,
}
pub enum Focusable {
    FileList,
    Dir,
    Search,
}

pub struct Interface {
    evt_rx: Receiver<Event>,
    pub audio_backend: Box<dyn Backend>,
    pub root: Option<Rc<TreeNode>>,
    pub file_list: Vec<ListElement>,
    pub expanded: HashSet<String>,
    pub list_offset: usize,
    pub input: String,
    pub focus: Focusable,
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
            thread::sleep(Duration::from_millis(200));
        });

        let path = std::env::current_dir().expect("Could not get current dir.");

        let mut interface = Self {
            audio_backend: backends::load(),
            evt_rx,
            root: None,
            file_list: vec![],
            expanded: HashSet::new(),
            list_offset: 0,
            focus: Focusable::FileList,
            input: String::new(),
        };
        interface.set_root(&path);

        interface
    }

    pub fn set_root(&mut self, path: &Path) {
        let path = PathBuf::from(path);
        let root = Rc::new(TreeNode::Folder(path.to_folder()));
        self.expanded.insert(root.key().clone());
        self.root = Some(root);
        self.rebuild_file_list();
    }

    pub fn rebuild_file_list(&mut self) {
        self.file_list = match &self.root {
            Some(root) => root.flatten(&self.expanded),
            _ => vec![],
        };
    }

    pub fn render_loop(&mut self) -> Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = TuiTerminal::new(backend)?;

        let mut list_state = ListState::default();
        let mut height = 0;

        loop {
            terminal.draw(|f| {
                height = f.size().height;

                let v_constraints = match self.focus {
                    Focusable::Dir | Focusable::Search => {
                        vec![Constraint::Length(3), Constraint::Min(1)]
                    }
                    _ => vec![Constraint::Min(1)],
                };

                let v_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(v_constraints)
                    .split(f.size());

                match self.focus {
                    Focusable::Dir | Focusable::Search => {
                        user_input::render(self, &v_chunks[0], f);
                    }
                    _ => {}
                }

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Length(50), Constraint::Min(1)])
                    .split(v_chunks[v_chunks.len() - 1]);

                let list = file_list::render_file_list(&self, &self.file_list, height as usize);

                f.render_stateful_widget(list, chunks[0], &mut list_state);
                player_state::render(self, &chunks[1], f);
            })?;

            match self.evt_rx.recv()? {
                Event::Input(key) => match key {
                    Key::Char('q') => {
                        drop(terminal);
                        std::process::exit(0);
                    }
                    _ => match self.focus {
                        Focusable::FileList => {
                            file_list::handle_input(self, &mut list_state, key, height as usize)
                        }
                        Focusable::Dir | Focusable::Search => user_input::handle_input(self, key),
                    },
                },
                _ => {}
            }
        }
    }
}
