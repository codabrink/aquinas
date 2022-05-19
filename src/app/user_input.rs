use std::path::PathBuf;

use super::*;
use tui::{
  layout::Rect,
  style::{Color, Style},
  terminal::Frame,
  widgets::{Block, Borders, Paragraph},
};

pub fn render<'a, B: Backend>(state: &'a mut App, area: Rect, frame: &mut Frame<B>) {
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

pub fn handle_input<'a>(state: &'a mut App, key: &KeyEvent) {
  match key.code {
    KeyCode::Backspace => {
      state.input.pop();
    }
    KeyCode::Enter => {
      process_cmd(state);
    }
    KeyCode::Char(c) => {
      state.input.push(c);
    }
    KeyCode::Esc => state.focus = Focusable::FileList,
    _ => {}
  }

  if state.focus == Focusable::Search {
    state.message(AppCommand::Select(0));
    state.library.search(&state.input);
  }
}

pub fn process_cmd<'a>(state: &'a mut App) {
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

        let mut meta = Meta::load().unwrap_or_default();
        meta.last_path = path.into();
        let _ = meta.save();
      }
    }
    Focusable::Search => {
      if let Some(node) = state.highlighted() {
        state.library.search("");
        state.play_path(&node.path);
      }
    }
    _ => {}
  }

  state.input = String::new();
  state.focus = Focusable::FileList;
}
