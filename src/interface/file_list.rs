use tui::widgets::List;

use super::Interface;
use crate::prelude::*;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, ListItem},
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

    List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("File List"))
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
