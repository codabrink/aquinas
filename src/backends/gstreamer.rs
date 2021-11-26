use gst::ClockTime;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_player as gst_player;
use std::path::{Path, PathBuf};
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

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
    fn last_played(&self) -> &Option<PathBuf> {
        &self.last_played
    }

    fn duration(path: &Path) -> u64 {
        let timeout: ClockTime = ClockTime::from_seconds(1);
        if let Ok(discoverer) = gst_pbutils::Discoverer::new(timeout) {
            if let Ok(info) = discoverer.discover_uri(&format!("file:///{}", path.display())) {
                if let Some(d) = info.duration() {
                    return d.seconds();
                }
            }
        }
        0
    }

    fn play(&mut self, path: &Path) {
        self.player.set_uri(&format!("file:///{}", path.display()));
        self.player.play();
        self.last_played = Some(path.to_owned());
        self.paused = false;
    }
    fn pause(&mut self) {
        self.player.pause();
        self.paused = true;
    }
    fn is_paused(&self) -> bool {
        self.paused
    }

    fn toggle(&mut self) {
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
