mod app;
mod backends;
mod config;
mod controls;
mod library;
mod meta;
#[cfg(feature = "metadata")]
mod metadata;
mod prelude;

pub use backends::Backend as AudioBackend;
pub use config::Config;
pub use library::{Library, Node};
pub use meta::Meta;
#[cfg(feature = "metadata")]
pub use metadata::{get_metadata, Metadata};
pub use prelude::*;
#[cfg(target_os = "macos")]
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
  let create_instance = || {
    let _ = app::App::new().run_app();
  };

  #[cfg(target_os = "macos")]
  {
    std::thread::spawn(create_instance);
    create_window();
  }
  #[cfg(not(target_os = "macos"))]
  create_instance();
}

// OSX is weird and requires a window to take media key events
// so let's make an invisible one
#[cfg(target_os = "macos")]
fn create_window() {
  let event_loop = EventLoop::new();

  let _window = WindowBuilder::new()
    .with_title("Aquinas Media Player")
    .with_visible(false)
    .build(&event_loop)
    .unwrap();

  event_loop.run(move |_event, _, control_flow| {
    control_flow.set_wait();
  });
}
