mod backends;
mod index;
mod interface;
mod metadata;
mod prelude;

fn main() {
  let _ = interface::Interface::new().render_loop();
}
