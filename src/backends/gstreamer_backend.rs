use anyhow::Result;
use gst::ClockTime;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_player as gst_player;
use std::path::{Path, PathBuf};

pub struct GStreamer {
  player: gst_player::Player,
  paused: bool,
  pub last_played: Option<PathBuf>,
}

impl super::Backend for GStreamer {
  fn new() -> Self {
    gst::init().expect("Could not initialize GStreamer.");
    let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
    let player = gst_player::Player::new(
      None,
      Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
    );

    Self {
      player,
      paused: true,
      last_played: None,
    }
  }
  fn last_played(&self) -> Option<&PathBuf> {
    self.last_played.as_ref()
  }

  fn track_finished(&self) -> bool {
    let progress = self.progress();
    !self.paused && progress.2 == 0 || progress.1 >= progress.2
  }

  fn play(&mut self, path: Option<&Path>) -> Result<()> {
    if let Some(path) = path {
      self
        .player
        .set_uri(Some(&format!("file:///{}", path.display())));
      self.last_played = Some(path.to_owned());
    }
    self.player.play();
    self.paused = false;
    Ok(())
  }
  fn pause(&mut self) {
    self.player.pause();
    self.paused = true;
  }
  fn is_paused(&self) -> bool {
    self.paused
  }

  fn play_pause(&mut self) {
    match self.paused {
      true => self.player.play(),
      false => self.player.pause(),
    }
    self.paused = !self.paused;
  }

  fn seek(&mut self, time: u64) {
    self.player.seek(ClockTime::from_seconds(time))
  }

  fn seek_delta(&mut self, delta_time: i64) {
    let time_pos = match self.player.position() {
      Some(t) => ClockTime::seconds(t) as i64,
      None => 0,
    };

    self.seek((time_pos + delta_time).max(0) as u64)
  }

  fn progress(&self) -> (f64, u64, u64) {
    let time_pos = match self.player.position() {
      Some(t) => ClockTime::seconds(t),
      None => 0,
    };

    let duration = match self.player.duration() {
      Some(d) => ClockTime::seconds(d),
      None => 119,
    };
    let percent = time_pos as f64 / (duration as f64);
    (percent, time_pos, duration)
  }
}
