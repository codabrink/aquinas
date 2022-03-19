use crate::prelude::*;
use audiotags::Tag;
use lewton::inside_ogg::OggStreamReader;
use std::fs;

#[derive(PartialEq, Default, Clone, Debug)]
pub struct Metadata {
  pub title: Option<String>,
  pub artist: Option<String>,
  pub album: Option<String>,
  pub track_number: Option<u16>,
}

pub fn get_metadata(path: &Path) -> Option<Metadata> {
  match path.extension() {
    Some(ext) => {
      let ext = ext.to_string_lossy().to_lowercase();
      match ext.as_str() {
        "ogg" => vorbis(path),
        _ => other(path),
      }
      .ok()
    }
    None => None,
  }
}

fn other(path: &Path) -> Result<Metadata> {
  let md = Tag::new().read_from_path(path).map(|t| Metadata {
    title: t.title().map(|t| t.to_owned()),
    artist: t.artist().map(|t| t.to_owned()),
    album: None, // handle later
    track_number: t.track().0,
  })?;
  Ok(md)
}

fn vorbis(path: &Path) -> Result<Metadata> {
  let file = fs::File::open(path)?;
  let source = OggStreamReader::new(file)?;

  let mut metadata = Metadata::default();

  for (k, v) in source.comment_hdr.comment_list {
    match k.to_lowercase().as_str() {
      "title" => metadata.title = Some(v),
      "artist" => metadata.artist = Some(v),
      "album" => metadata.album = Some(v),
      "tracknumber" => metadata.track_number = v.parse().ok(),
      _ => {}
    }
  }

  Ok(metadata)
}
