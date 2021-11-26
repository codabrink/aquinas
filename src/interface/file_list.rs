use super::*;
use crate::prelude::*;
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
    let list_items: Vec<ListItem> = state.file_list
        [state.list_offset..(state.list_offset + area.height as usize).min(state.file_list.len())]
        .iter()
        .map(|tn| render_list_item(state, tn))
        .collect();

    let mut title = match &state.root {
        Some(root) => root.title.clone(),
        _ => String::from("No Folder"),
    };
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

fn render_list_item<'a>(state: &'a Interface, el: &'a TreeNode) -> ListItem<'a> {
    ListItem::new(match el.children {
        Some(_) => Spans::from(vec![
            Span::from(" ".repeat(el.depth * 2)),
            Span::from(match state.expanded.contains(&el.key) {
                true => "▼ ",
                false => "▶ ",
            }),
            Span::styled(
                &el.title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        None => Spans::from(vec![
            Span::from(" ".repeat(el.depth * 2)),
            Span::from(el.title.as_ref()),
        ]),
    })
}

pub fn handle_input(state: &mut Interface, list_state: &mut ListState, key: Key, height: usize) {
    match key {
        Key::Down | Key::Ctrl('n') => {
            state.list_index = (state.list_index + 1).min(state.file_list.len().saturating_sub(1));
            state.list_offset = state
                .list_offset
                .max(state.list_index.saturating_sub(height.saturating_sub(1)));
            list_state.select(Some(state.list_index.saturating_sub(state.list_offset)));
        }
        Key::Up | Key::Ctrl('p') => {
            state.list_index = state.list_index.saturating_sub(1);
            state.list_offset = state.list_offset.min(state.list_index);
            list_state.select(Some(state.list_index.saturating_sub(state.list_offset)));
        }
        Key::Right | Key::Ctrl('f') => {
            if let Some(i) = list_state.selected() {
                let i = i + state.list_offset;
                if let Some(tn) = state.file_list.get(i) {
                    if let Some(_) = tn.children {
                        state.expanded.insert(tn.key.clone());
                        state.rebuild_file_list();
                    }
                }
            }
        }
        Key::Left | Key::Ctrl('b') => {
            if let Some(i) = list_state.selected() {
                let i = i + state.list_offset;
                if let Some(tn) = state.file_list.get(i) {
                    if let Some(_) = tn.children {
                        state.expanded.remove(&tn.key);
                        state.rebuild_file_list();
                    }
                }
            }
        }
        Key::Char('f') => {
            state.audio_backend.seek_delta(2);
        }
        Key::Char('F') => {
            state.audio_backend.seek_delta(5);
        }
        Key::Char('b') => {
            state.audio_backend.seek_delta(-2);
        }
        Key::Char('B') => {
            state.audio_backend.seek_delta(-5);
        }
        Key::Char('\n') => {
            if let Some(i) = list_state.selected() {
                let i = i + state.list_offset;
                if let Some(tn) = state.file_list.get(i) {
                    if tn.children.is_none() {
                        state.audio_backend.play(&tn.path);
                    }
                }
            }
        }
        Key::Char(' ') => state.audio_backend.toggle(),
        Key::Char('d') => state.focus = Focusable::Dir,
        Key::Char('s') => state.focus = Focusable::Search,
        _ => {}
    }
}
