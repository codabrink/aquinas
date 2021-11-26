use super::*;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
};

pub fn render<'a>(state: &'a mut Interface, area: &Rect, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(2)])
        .split(*area);

    let (pct, pos, dur) = state.progress;
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::TOP))
        .gauge_style(Style::default().fg(Color::Blue))
        .percent((pct * 100.) as u16)
        .label(format!("{}/{}", pos, dur));

    frame.render_widget(gauge, chunks[1]);
}
