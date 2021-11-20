use super::{Focusable, Interface};
use crate::prelude::*;
use termion::event::Key;
use tui::widgets::List;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, ListItem, ListState},
};

pub fn render_file_list<'a>(
    state: &'a Interface,
    file_list: &'a [ListElement],
    height: usize,
) -> List<'a> {
    let list_items: Vec<ListItem> = file_list
        [state.list_offset..(state.list_offset + height).min(file_list.len())]
        .iter()
        .map(|tn| render_list_item(state, tn))
        .collect();

    let mut title = match &state.root {
        Some(root) => root.file(),
        _ => String::from("No Folder"),
    };
    title.push_str(&" ".repeat(80 - title.len()));

    List::new(list_items)
        .block(
            Block::default()
                // .borders(Borders::RIGHT)
                .border_style(Style::default().fg(match state.focus {
                    Focusable::FileList => Color::Green,
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
        )
}

fn render_list_item<'a>(state: &'a Interface, el: &'a ListElement) -> ListItem<'a> {
    ListItem::new(match el.is_folder {
        true => Spans::from(vec![
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
        false => Spans::from(vec![
            Span::from(" ".repeat(el.depth * 2)),
            Span::from(el.title.as_ref()),
        ]),
    })
}

pub fn handle_input(state: &mut Interface, list_state: &mut ListState, key: Key, height: usize) {
    match key {
        Key::Down | Key::Ctrl('n') => {
            let i = match list_state.selected() {
                Some(i) => {
                    let height = height as usize;
                    if i == height {
                        state.list_offset = (state.list_offset + 1)
                            .min(state.file_list.len().saturating_sub(height));
                    }
                    (i + 1).min(height).min(state.file_list.len() - 1)
                }
                None => 0,
            };
            list_state.select(Some(i));
        }
        Key::Up | Key::Ctrl('p') => {
            let i = match list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        state.list_offset = state.list_offset.saturating_sub(1)
                    }
                    i.saturating_sub(1)
                }
                None => 0,
            };
            list_state.select(Some(i));
        }
        Key::Right | Key::Ctrl('f') => {
            if let Some(i) = list_state.selected() {
                let i = i + state.list_offset;
                if let Some(el) = state.file_list.get(i) {
                    if let Some(tn) = el.tn.upgrade() {
                        if let TreeNode::Folder(f) = &*tn {
                            state.expanded.insert(f.key.clone());
                            state.rebuild_file_list();
                        }
                    }
                }
            }
        }
        Key::Left | Key::Ctrl('b') => {
            if let Some(i) = list_state.selected() {
                let i = i + state.list_offset;
                if let Some(el) = state.file_list.get(i) {
                    if let Some(tn) = el.tn.upgrade() {
                        if let TreeNode::Folder(f) = &*tn {
                            state.expanded.remove(&f.key);
                            state.rebuild_file_list();
                        }
                    }
                }
            }
        }
        Key::Char('\n') => {
            if let Some(i) = list_state.selected() {
                let i = i + state.list_offset;
                if let Some(el) = state.file_list.get(i) {
                    if let Some(tn) = el.tn.upgrade() {
                        if let TreeNode::File(f) = &*tn {
                            state.audio_backend.play(&f.path);
                        }
                    }
                }
            }
        }
        Key::Char(' ') => state.audio_backend.toggle(),
        Key::Char('d') => state.focus = Focusable::Dir,
        _ => {}
    }
}
