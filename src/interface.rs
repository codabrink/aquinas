use crate::prelude::*;
use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver};
use std::{collections::VecDeque, io, path::PathBuf, thread, time::Duration};
use termion::{event::Key, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal as TuiTerminal,
};

enum Event {
    Input(Key),
    Tick,
}

pub struct Interface {
    evt_rx: Receiver<Event>,
    root: Folder,
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
        }
    }

    pub fn render_loop(&mut self) -> Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = TuiTerminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![]);
            })?;

            match self.evt_rx.recv()? {
                Event::Input(key) => {
                    match key {
                        Key::Ctrl('n') => {
                            // next song
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
