mod gstreamer;
use std::boxed::Box;
use std::path::Path;

pub fn load() -> Box<dyn Backend> {
    // in the future this will be configurable,
    // but for now we only have one backend.

    Box::new(gstreamer::GStreamer::new())
}

pub trait Backend {
    fn new() -> Self
    where
        Self: Sized;
    fn duration(path: &Path) -> u64
    where
        Self: Sized;
    fn play(&mut self, path: &Path);
    fn progress(&mut self) -> (f64, u64, u64);
}
