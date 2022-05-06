use crate::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct Meta {
  pub last_path: Option<PathBuf>,
}

impl Meta {
  fn config_dir() -> Result<PathBuf> {
    let config_dir = dirs::data_local_dir()
      .expect("Cannot save meta info")
      .join("aquinas");

    Ok(config_dir)
  }

  pub fn save(&self) -> Result<()> {
    let config_dir = Self::config_dir()?;
    let _ = std::fs::create_dir_all(&config_dir);
    let _ = std::fs::write(config_dir.join("meta.toml"), toml::to_string(self)?);

    Ok(())
  }

  pub fn load() -> Result<Self> {
    let config_dir = Self::config_dir()?;
    let serialized = std::fs::read_to_string(config_dir.join("meta.toml"))?;
    let meta = toml::from_str(&serialized)?;
    Ok(meta)
  }
}
