use crate::prelude::*;
use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver};
use std::{collections::VecDeque, io, ops::Add, path::PathBuf, thread, time::Duration};
use termion::{event::Key, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal as TuiTerminal,
};

enum Event {
    Input(Key),
    Tick,
}

pub struct Interface {
    evt_rx: Receiver<Event>,
    root: Folder,
    expanded: HashSet<String>,
    list_offset: usize,
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

        Self {
            evt_rx,
            root: path.to_folder(),
            expanded: HashSet::new(),
            list_offset: 0,
        }
    }

    pub fn render_loop(&mut self, audio_backend: &mut Box<dyn Backend>) -> Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = TuiTerminal::new(backend)?;

        let file_list = self.root.flatten(&self.expanded);
        let rendered_file_list: Vec<ListItem> = file_list
            .iter()
            .map(|tn| ListItem::new(Span::from(String::from(tn))))
            .collect();
        let mut list_state = ListState::default();

        let mut height = 0;

        loop {
            terminal.draw(|f| {
                height = f.size().height;

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(34), Constraint::Percentage(66)])
                    .split(f.size());
                let a = f.size().height;

                f.render_stateful_widget(
                    List::new(Vec::from(
                        &rendered_file_list[self.list_offset
                            ..(self.list_offset + height as usize).min(rendered_file_list.len())],
                    ))
                    .highlight_style(
                        Style::default()
                            .bg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    ),
                    chunks[0],
                    &mut list_state,
                );
            })?;

            match self.evt_rx.recv()? {
                Event::Input(key) => {
                    match key {
                        Key::Down => {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    let height = height as usize;
                                    if i == height {
                                        self.list_offset = (self.list_offset + 1)
                                            .min(rendered_file_list.len().saturating_sub(height));
                                    }
                                    (i + 1).min(height)
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));
                        }
                        Key::Up => {
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
                        Key::Ctrl('n') => {
                            // next song
                        }
                        Key::Char('\n') => {
                            if let Some(i) = list_state.selected() {
                                let i = i + self.list_offset;
                                if let Some(tn) = file_list.get(i) {
                                    match tn {
                                        BorrowedTreeNode::File(f) => {
                                            audio_backend.play(&f.path);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Key::Char('q') => {
                            drop(terminal);
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }
    }
}
