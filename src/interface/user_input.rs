use std::path::PathBuf;

use super::*;
use tui::{
  layout::Rect,
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};

pub fn render<'a>(state: &'a mut Interface, area: Rect, frame: &mut Frame) {
  let paragraph = Paragraph::new(state.input.as_ref()).block(
    Block::default()
      .borders(Borders::ALL)
      .border_style(Style::default().fg(Color::Blue))
      .title(match state.focus {
        Focusable::Dir => "Change Directory",
        Focusable::Search => "Search",
        _ => "",
      }),
  );
  frame.render_widget(paragraph, area);
}

pub fn handle_input<'a>(state: &'a mut Interface, key: Key) {
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

pub fn process_cmd<'a>(state: &'a mut Interface) {
  match state.focus {
    Focusable::Dir => {
      let mut input = state.input.clone();
      if let Some(home_dir) = dirs::home_dir() {
        input = input.replace('~', &home_dir.display().to_string());
      }
      let input_str = input.as_str().trim();

      let path = match (input_str, &state.library.root) {
        ("..", root) => {
          let parent = match root.path.parent() {
            Some(parent) => parent,
            _ => &root.path,
          };
          PathBuf::from(parent)
        }
        _ => PathBuf::from(input),
      };

      if path.is_dir() {
        state.set_root(&path);
      }
    }
    _ => {}
  }

  state.input = String::new();
  state.focus = Focusable::FileList;
}
