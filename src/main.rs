mod backends;
mod interface;
mod library;
mod metadata;
mod prelude;

pub use backends::Backend;
pub use library::{Library, Node};
pub use prelude::*;

fn main() {
  let _ = interface::Interface::new().render_loop();
}
