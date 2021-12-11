mod backends;
mod interface;
mod prelude;

fn main() {
  let _ = interface::Interface::new().render_loop();
}
