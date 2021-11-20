use super::Interface;
use crate::prelude::*;
use tui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

pub fn render<'a>(state: &'a mut Interface) -> Paragraph<'a> {
    Paragraph::new(state.cmd.as_ref()).block(Block::default().borders(Borders::ALL).title("Cmd"))
}
