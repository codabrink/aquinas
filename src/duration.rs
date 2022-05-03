use ogg_metadata::OggFormat;

use crate::*;

pub fn duration(path: &Path) -> Result<Option<u64>> {
  let file = std::fs::File::open(path)?;
  let secs = match extension(path) {
    Some("mp3") => mp3_duration::from_file(&file).ok().map(|d| d.as_secs()),
    Some("ogg") => {
      let fmt = ogg_metadata::read_format(&file)?;
      match fmt.first() {
        Some(OggFormat::Vorbis(meta)) => {
          meta.length_in_samples.map(|s| s / meta.sample_rate as u64)
        }
        // TODO: test
        Some(OggFormat::Opus(meta)) => meta.length_in_48khz_samples.map(|s| s / 48_000_000),
        _ => None,
      }
    }
    _ => None,
  };
  Ok(secs)
}
