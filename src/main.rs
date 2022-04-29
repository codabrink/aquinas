mod backends;
mod interface;
mod library;
mod metadata;
mod prelude;

pub use backends::Backend as AudioBackend;
pub use library::{Library, Node};
pub use metadata::{get_metadata, Metadata};
pub use prelude::*;

#[tokio::main]
async fn main() {
  let _ = interface::App::new().run_app();
}
