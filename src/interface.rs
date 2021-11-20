mod cmd_input;
mod file_list;

use crate::prelude::*;
use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver};
use std::{collections::VecDeque, io, ops::Add, path::PathBuf, rc::Rc, thread, time::Duration};
use termion::{event::Key, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal as TuiTerminal,
};

enum Event {
    Input(Key),
    Tick,
}

pub struct Interface {
    evt_rx: Receiver<Event>,
    pub root: Rc<TreeNode>,
    pub expanded: HashSet<String>,
    pub list_offset: usize,
    pub cmd: String,
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
        let root = Rc::new(TreeNode::Folder(path.to_folder()));
        let mut expanded = HashSet::new();
        expanded.insert(root.key().clone());

        Self {
            evt_rx,
            root,
            expanded,
            list_offset: 0,
            cmd: String::new(),
        }
    }

    pub fn render_loop(&mut self, audio_backend: &mut Box<dyn Backend>) -> Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = TuiTerminal::new(backend)?;

        let mut file_list = self.root.flatten(&self.expanded);
        let mut list_state = ListState::default();
        let mut height = 0;
        let mut rebuild = false;
        let mut show_cmd = false;

        loop {
            if rebuild {
                file_list = self.root.flatten(&self.expanded);
                rebuild = false;
            }

            terminal.draw(|f| {
                height = f.size().height;

                let v_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                    .split(f.size());

                f.render_widget(cmd_input::render(self), v_chunks[0]);

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Length(80), Constraint::Min(1)])
                    .split(v_chunks[1]);

                let list = file_list::render_file_list(&self, &file_list, height as usize);

                f.render_stateful_widget(list, chunks[0], &mut list_state);
            })?;

            match self.evt_rx.recv()? {
                Event::Input(key) => match key {
                    Key::Down | Key::Ctrl('n') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                let height = height as usize;
                                if i == height {
                                    self.list_offset = (self.list_offset + 1)
                                        .min(file_list.len().saturating_sub(height));
                                }
                                (i + 1).min(height).min(file_list.len() - 1)
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    Key::Up | Key::Ctrl('p') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.list_offset = self.list_offset.saturating_sub(1)
                                }
                                i.saturating_sub(1)
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    Key::Right | Key::Ctrl('f') => {
                        if let Some(i) = list_state.selected() {
                            let i = i + self.list_offset;
                            if let Some(el) = file_list.get(i) {
                                if let Some(tn) = el.tn.upgrade() {
                                    if let TreeNode::Folder(f) = &*tn {
                                        self.expanded.insert(f.key.clone());
                                        rebuild = true;
                                    }
                                }
                            }
                        }
                    }
                    Key::Left | Key::Ctrl('b') => {
                        if let Some(i) = list_state.selected() {
                            let i = i + self.list_offset;
                            if let Some(el) = file_list.get(i) {
                                if let Some(tn) = el.tn.upgrade() {
                                    if let TreeNode::Folder(f) = &*tn {
                                        self.expanded.remove(&f.key);
                                        rebuild = true;
                                    }
                                }
                            }
                        }
                    }
                    Key::Char('\n') => {
                        if let Some(i) = list_state.selected() {
                            let i = i + self.list_offset;
                            if let Some(el) = file_list.get(i) {
                                if let Some(tn) = el.tn.upgrade() {
                                    if let TreeNode::File(f) = &*tn {
                                        audio_backend.play(&f.path);
                                    }
                                }
                            }
                        }
                    }
                    Key::Char(' ') => audio_backend.toggle(),
                    Key::Char('c') => show_cmd = !show_cmd,
                    Key::Char('q') => {
                        drop(terminal);
                        std::process::exit(0);
                    }
                    _ => {}
                },

                _ => {}
            }
        }
    }
}
