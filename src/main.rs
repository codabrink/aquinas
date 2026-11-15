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
use clap::Parser;
#[cfg(target_os = "macos")]
use winit::{
  application::ApplicationHandler,
  event::WindowEvent,
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  window::{Window, WindowId},
};

#[derive(Parser)]
#[command(version, about)]
struct Args {
  /// Directory to open (defaults to the current directory)
  directory: Option<PathBuf>,
}

fn main() {
  let args = Args::parse();
  let create_instance = move || {
    let _ = app::App::new(args.directory).run_app();
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
  let event_loop = EventLoop::new().unwrap();
  event_loop.set_control_flow(ControlFlow::Wait);
  event_loop.run_app(&mut HiddenWindow::default()).unwrap();
}

#[cfg(target_os = "macos")]
#[derive(Default)]
struct HiddenWindow {
  window: Option<Window>,
}

#[cfg(target_os = "macos")]
impl ApplicationHandler for HiddenWindow {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let attributes = Window::default_attributes()
        .with_title("Aquinas Media Player")
        .with_visible(false);
      self.window = Some(event_loop.create_window(attributes).unwrap());
    }
  }

  fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}
