#![feature(in_band_lifetimes)]

use std::fs;

mod backends;
mod interface;
mod prelude;

use crate::prelude::*;

fn main() {
    let mut backend = backends::load();
    // if let Err(e) = play_first(&mut backend) {
    // println!("{:?}", e);
    // };

    let mut interface = interface::Interface::new();
    interface.render_loop(&mut backend);
}

fn play_first(backend: &mut Box<dyn Backend>) -> Result<()> {
    let path = std::env::current_dir()?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "ogg" {
                backend.play(&path);

                break;
            }
        }
    }

    Ok(())
}
