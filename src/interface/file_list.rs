use super::*;
use termion::event::Key;
use tui::layout::Rect;
use tui::widgets::List;
use tui::{
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, ListItem, ListState},
};

pub fn render_file_list<'a>(
  state: &'a Interface,
  list_state: &mut ListState,
  area: Rect,
  frame: &mut Frame,
) {
  let list_items: Vec<ListItem> = state.library.file_list[state.list_offset
    ..(state.list_offset + area.height as usize).min(state.library.file_list.len())]
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

fn render_list_item<'a>(state: &'a Interface, node: &'a Element, depth: usize) -> ListItem<'a> {
  ListItem::new(match (node.is_dir(), state.backend.last_played()) {
    (true, _) => Spans::from(vec![
      Span::from(" ".repeat(depth * 2)),
      Span::from(match state.library.open_dirs.contains_key(node.path()) {
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
    (false, Some(lp)) if *lp == node.path() => Spans::from(vec![Span::styled(
      format!("{}{}", " ".repeat(depth * 2), node.title()),
      Style::default().bg(Color::White).fg(Color::Black),
    )]),
    _ => Spans::from(vec![
      Span::from(" ".repeat(depth * 2)),
      Span::from(node.title().as_ref()),
    ]),
  })
}

pub fn handle_input<'a>(
  state: &'a mut Interface,
  list_state: &mut ListState,
  key: Key,
  height: usize,
) {
  match key {
    Key::Down | Key::Ctrl('n') => {
      state.list_index =
        (state.list_index + 1).min(state.library.file_list.len().saturating_sub(1));
      state.list_offset = state
        .list_offset
        .max(state.list_index.saturating_sub(height.saturating_sub(3)));
      list_state.select(Some(state.list_index.saturating_sub(state.list_offset)));
    }
    Key::Up | Key::Ctrl('p') => {
      state.list_index = state.list_index.saturating_sub(1);
      state.list_offset = state.list_offset.min(state.list_index);
      list_state.select(Some(state.list_index.saturating_sub(state.list_offset)));
    }
    Key::Right | Key::Ctrl('f') => {
      if let Some(i) = list_state.selected() {
        state.expand(i + state.list_offset);
      }
    }
    Key::Left | Key::Ctrl('b') => {
      if let Some(i) = list_state.selected() {
        state.collapse(i + state.list_offset);
      }
    }
    Key::Char('f') => {
      state.backend.seek_delta(2);
    }
    Key::Char('F') => {
      state.backend.seek_delta(5);
      ()
    }
    Key::Char('b') => {
      state.backend.seek_delta(-2);
    }
    Key::Char('B') => {
      state.backend.seek_delta(-5);
    }
    Key::Char('\n') => {
      state.play(state.list_index);
      // if let Some(root) = &state.root {
      // state.backend.tags(&root.path);
      // }
    }
    Key::Char(' ') => state.backend.toggle(),
    Key::Char('d') => state.focus = Focusable::Dir,
    Key::Char('s') => state.focus = Focusable::Search,
    _ => {}
  }
}
