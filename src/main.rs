mod app;
mod backends;
#[cfg(feature = "rodio_backend")]
mod duration;
mod library;
mod metadata;
mod prelude;

pub use backends::Backend as AudioBackend;
pub use library::{Library, Node};
pub use metadata::{get_metadata, Metadata};
pub use prelude::*;

#[tokio::main]
async fn main() {
  let mut app = app::App::new();
  let result = app.run_app();
  println!("{:?}", result);
}
