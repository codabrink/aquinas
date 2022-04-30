mod app;
mod backends;
mod library;
mod metadata;
mod prelude;

pub use backends::Backend as AudioBackend;
pub use library::{Library, Node};
pub use metadata::{get_metadata, Metadata};
pub use prelude::*;

#[tokio::main]
async fn main() {
  let _ = app::App::new().run_app();
}
