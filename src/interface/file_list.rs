use tui::widgets::List;

use super::Interface;
use crate::prelude::*;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, ListItem},
};

pub fn render_file_list(
    state: &'a Interface,
    file_list: &'a [BorrowedTreeNode],
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

fn render_list_item(state: &'a Interface, tn: &'a BorrowedTreeNode) -> ListItem<'a> {
    ListItem::new(match tn {
        BorrowedTreeNode::Folder(f) => Spans::from(vec![
            Span::from(" ".repeat(f.depth * 2)),
            Span::from(match state.expanded.contains(&f.path_string) {
                true => "▼ ",
                false => "▶ ",
            }),
            Span::styled(
                tn.file(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        BorrowedTreeNode::File(f) => Spans::from(vec![
            Span::from(" ".repeat(f.depth * 2)),
            Span::from(tn.file()),
        ]),
    })
}
