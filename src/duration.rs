use crate::*;
use ogg_metadata::OggFormat;
use rodio::{Decoder, Source};

pub fn duration(path: &Path) -> Result<u64> {
  let file = std::fs::File::open(path)?;
  let ext = extension(path);
  if let Some(ext) = ext {
    if ext == "mp3" {
      if let Ok(dur) = mp3_duration::from_file(&file) {
        return Ok(dur.as_secs());
      }
    }
    if ext == "ogg" {
      if let Ok(fmt) = ogg_metadata::read_format(&file) {
        for fmt in fmt {
          if let OggFormat::Vorbis(meta) = &fmt {
            if let Some(length) = meta.length_in_samples {
              return Ok(length / meta.sample_rate as u64);
            }
          }
          if let OggFormat::Opus(meta) = fmt {
            if let Some(length) = meta.length_in_48khz_samples {
              // Todo: test this
              return Ok(length / 48_000_000);
            }
          }
        }
      }
    }
  }

  // Last resort
  // Get it the honky and expensive way - count the samples
  duration_from_samples(path)
}

fn duration_from_samples(path: &Path) -> Result<u64> {
  let file = File::open(path)?;
  let source = Decoder::new(BufReader::new(file))?;

  let sample_rate = source.sample_rate();
  let channels = source.channels();
  let samples = source.count();

  Ok(samples as u64 / sample_rate as u64 / channels as u64)
}
