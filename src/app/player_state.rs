use super::*;
use tui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  terminal::Frame,
  widgets::{Block, Borders, Gauge},
};

pub fn render<'a, B: Backend>(state: &'a mut App, area: &Rect, frame: &mut Frame<B>) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![Constraint::Length(2)])
    .split(*area);

  let (pct, pos, dur) = state.progress;

  let pos_min = pos / 60;
  let pos_sec = pos % 60;
  let dur_min = dur / 60;
  let dur_sec = dur % 60;

  let gauge = Gauge::default()
    .block(Block::default().borders(Borders::TOP))
    .gauge_style(Style::default().fg(Color::Blue))
    .percent((pct * 100.) as u16)
    .label(format!(
      "{}{}:{:0>2}/{}:{:0>2}",
      state
        .playing
        .as_ref()
        .map(|p| format!("{} - ", p))
        .unwrap_or(String::new()),
      pos_min,
      pos_sec,
      dur_min,
      dur_sec
    ));

  frame.render_widget(gauge, chunks[0]);
}
