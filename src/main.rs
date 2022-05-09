#[macro_use]
extern crate lazy_static;

mod app;
mod backends;
mod config;
#[cfg(feature = "rodio_backend")]
mod duration;
mod library;
mod meta;
#[cfg(feature = "metadata")]
mod metadata;
mod mpris;
mod prelude;

pub use backends::Backend as AudioBackend;
pub use config::Config;
pub use library::{Library, Node};
pub use meta::Meta;
#[cfg(feature = "metadata")]
pub use metadata::{get_metadata, Metadata};
pub use prelude::*;

fn main() {
  let mut app = app::App::new();
  let result = app.run_app();
  println!("{:?}", result);
}
