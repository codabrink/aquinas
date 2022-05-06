use crate::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
  pub scan_depth_limit: usize,
}

impl ::std::default::Default for Config {
  fn default() -> Self {
    Self {
      scan_depth_limit: 12,
    }
  }
}

impl Config {
  fn config_dir() -> Result<PathBuf> {
    Meta::config_dir()
  }

  pub fn save(&self) -> Result<()> {
    let config_dir = Self::config_dir()?;
    let _ = std::fs::create_dir_all(&config_dir);
    let _ = std::fs::write(config_dir.join("config.toml"), toml::to_string(self)?);

    Ok(())
  }

  pub fn load() -> Result<Self> {
    let config_dir = Self::config_dir()?;
    let serialized = std::fs::read_to_string(config_dir.join("config.toml"))?;
    Ok(toml::from_str(&serialized)?)
  }
}
