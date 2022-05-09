use super::*;
use crossterm::event::KeyCode;
use tui::layout::Rect;
use tui::widgets::List;
use tui::{
  style::{Color, Modifier, Style},
  terminal::Frame,
  text::{Span, Spans},
  widgets::{Block, Borders, ListItem, ListState},
};

pub fn render_file_list<'a, B: Backend>(
  state: &'a mut App,
  area: Rect,
  frame: &mut Frame<B>,
  list_state: &mut ListState,
) {
  let list_items: Vec<ListItem> = state
    .library
    .file_list()
    .get_range(&state.view_range())
    .into_iter()
    .map(|(node, depth)| render_list_item(state, node, *depth))
    .collect();

  let mut title = state.library.root.title().to_owned();
  title.push_str(&" ".repeat(area.width as usize - title.len()));

  let list = List::new(list_items)
    .block(
      Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(match state.focus {
          Focusable::FileList => Color::White,
          _ => Color::White,
        }))
        .title(Span::styled(
          title,
          Style::default()
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        )),
    )
    .highlight_style(
      Style::default()
        .bg(Color::LightGreen)
        .add_modifier(Modifier::BOLD),
    );
  frame.render_stateful_widget(list, area, list_state);
}

fn render_list_item<'a>(state: &'a App, node: &'a Node, depth: usize) -> ListItem<'a> {
  ListItem::new(match (node.is_dir(), state.backend.last_played()) {
    (true, _) => Spans::from(vec![
      Span::from(" ".repeat(depth * 2)),
      Span::from(match state.library.open_dirs.contains(&node.path) {
        true => "▼ ",
        false => "▶ ",
      }),
      Span::styled(
        node.title(),
        Style::default()
          .fg(Color::Cyan)
          .add_modifier(Modifier::BOLD),
      ),
    ]),
    (false, Some(lp)) if *lp == node.path => Spans::from(vec![Span::styled(
      format!("{}{}", " ".repeat(depth * 2), node.title()),
      Style::default().bg(Color::White).fg(Color::Black),
    )]),
    _ => Spans::from(vec![
      Span::from(" ".repeat(depth * 2)),
      Span::from(node.title().as_ref()),
    ]),
  })
}

pub fn handle_input<'a>(state: &'a mut App, key: &KeyEvent) {
  let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

  match (key.code, ctrl) {
    (KeyCode::Right, _) | (KeyCode::Char('f'), true) => {
      if let Some(selected) = state.selected {
        state.expand(selected);
      }
    }
    (KeyCode::Left, _) | (KeyCode::Char('b'), true) => {
      if let Some(selected) = state.selected {
        state.collapse(selected);
      }
    }
    (KeyCode::Char('f'), _) => {
      state.backend.seek_delta(2);
    }
    (KeyCode::Char('F'), _) => {
      state.backend.seek_delta(5);
      ()
    }
    (KeyCode::Char('b'), _) => {
      state.backend.seek_delta(-2);
    }
    (KeyCode::Char('B'), _) => {
      state.backend.seek_delta(-5);
    }
    (KeyCode::Enter, _) => {
      if let Some(selected) = state.selected {
        state.play(selected);
      }
    }
    (KeyCode::Char(' '), _) => state.play_pause(),
    (KeyCode::Char('d'), _) => state.focus = Focusable::Dir,
    (KeyCode::Char('s'), _) => state.focus = Focusable::Search,
    _ => {}
  }
}
