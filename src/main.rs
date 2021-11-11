#![feature(in_band_lifetimes)]

mod backends;
mod interface;
mod prelude;

fn main() {
    let mut backend = backends::load();
    // if let Err(e) = play_first(&mut backend) {
    // println!("{:?}", e);
    // };

    let mut interface = interface::Interface::new();
    interface.render_loop(&mut backend);
}
