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
  let list_items: Vec<ListItem> = state.library.file_list()[state.list_offset
    ..(state.list_offset + area.height as usize).min(state.library.file_list().len())]
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

pub fn handle_input<'a>(state: &'a mut App, list_state: &mut ListState, key: &KeyEvent) {
  let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

  match key.code {
    KeyCode::Right | KeyCode::Char('f') if ctrl => {
      if let Some(i) = list_state.selected() {
        state.expand(i + state.list_offset);
      }
    }
    KeyCode::Left | KeyCode::Char('b') if ctrl => {
      if let Some(i) = list_state.selected() {
        state.collapse(i + state.list_offset);
      }
    }
    KeyCode::Char('f') => {
      state.backend.seek_delta(2);
    }
    KeyCode::Char('F') => {
      state.backend.seek_delta(5);
      ()
    }
    KeyCode::Char('b') => {
      state.backend.seek_delta(-2);
    }
    KeyCode::Char('B') => {
      state.backend.seek_delta(-5);
    }
    KeyCode::Char('\n') => {
      state.play(state.list_index);
    }
    KeyCode::Char(' ') => state.backend.toggle(),
    KeyCode::Char('d') => state.focus = Focusable::Dir,
    KeyCode::Char('s') => state.focus = Focusable::Search,
    _ => {}
  }
}
