use crate::app::AppCommand;
use crate::*;
use crossbeam_channel::{Receiver, Sender};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};

#[derive(Default)]
pub struct Metadata {
  pub title: Option<String>,
  pub artist: Option<String>,
  pub album: Option<String>,
}

impl<'a> From<&'a Metadata> for MediaMetadata<'a> {
  fn from(metadata: &'a Metadata) -> Self {
    Self {
      title: metadata.title.as_ref().map(|t| t.as_str()),
      album: metadata.album.as_ref().map(|a| a.as_str()),
      artist: metadata.artist.as_ref().map(|a| a.as_str()),
      ..Default::default()
    }
  }
}

pub enum PlaybackStatus {
  Playing(Option<Metadata>),
  Paused,
}

pub fn event_listener(command: Arc<Sender<AppCommand>>, play_status: Receiver<PlaybackStatus>) {
  #[cfg(not(target_os = "windows"))]
  let hwnd = None;

  #[cfg(target_os = "windows")]
  let hwnd = {
    use raw_window_handle::windows::WindowsHandle;

    let handle: WindowsHandle = unimplemented!();
    Some(handle.hwnd)
  };

  let config = PlatformConfig {
    dbus_name: "aquinas",
    display_name: "Aquinas",
    hwnd,
  };

  let mut controls = MediaControls::new(config).unwrap();

  controls
    .attach(move |event: MediaControlEvent| {
      use MediaControlEvent::*;
      match event {
        Toggle => {
          let _ = command.send(AppCommand::PlayPause);
        }
        Play => {
          let _ = command.send(AppCommand::Play(None));
        }
        Pause | Stop => {
          let _ = command.send(AppCommand::Pause);
        }
        Next => {
          let _ = command.send(AppCommand::Next);
        }
        Previous => {
          let _ = command.send(AppCommand::Prev);
        }
        _ => {}
      }
    })
    .unwrap();

  std::thread::spawn(move || {
    for status in play_status.iter() {
      let _ = match status {
        PlaybackStatus::Playing(Some(metadata)) => {
          let _ = controls.set_metadata((&metadata).into());
          controls.set_playback(MediaPlayback::Playing { progress: None })
        }
        PlaybackStatus::Playing(_) => {
          controls.set_playback(MediaPlayback::Playing { progress: None })
        }
        PlaybackStatus::Paused => controls.set_playback(MediaPlayback::Paused { progress: None }),
      };
    }
  });
}
