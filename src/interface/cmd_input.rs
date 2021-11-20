use std::path::PathBuf;

use super::*;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn render<'a>(state: &'a mut Interface) -> Paragraph<'a> {
    Paragraph::new(state.input.as_ref()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(match state.focus {
                Focusable::Dir => Color::Green,
                _ => Color::White,
            }))
            .title(match state.focus {
                Focusable::Dir => "Change Directory",
                _ => "",
            }),
    )
}

pub fn handle_input(state: &mut Interface, key: Key) {
    match key {
        Key::Backspace => {
            state.input.pop();
        }
        Key::Char('\n') => {
            process_cmd(state);
        }
        Key::Char(c) => {
            state.input.push(c);
        }
        Key::Esc => state.focus = Focusable::FileList,
        _ => {}
    }
}

fn process_cmd(state: &mut Interface) {
    match state.focus {
        Focusable::Dir => {
            let path = PathBuf::from(&state.input);
            if path.is_dir() {
                state.set_root(path);
            }
        }
        _ => {}
    }

    state.input = String::new();
    state.focus = Focusable::FileList;
}
