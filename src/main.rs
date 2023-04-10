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
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
  std::thread::spawn(|| {
    let mut app = app::App::new();
    app.run_app();
  });

  let event_loop = EventLoop::new();

  // OSX is weird and requires a window to take media key events
  // so let's make an invisible one
  let _window = WindowBuilder::new()
    .with_title("Aquinas Media Player")
    .with_visible(false)
    .build(&event_loop)
    .unwrap();

  event_loop.run(move |event, _, control_flow| {
    control_flow.set_wait();
  });
}
