use crate::publishers::cloudinary::CloudinaryConfig;
use crate::publishers::script::ScriptConfig;
use crate::UploadError;
use dirs::home_dir;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::Path;

const CONFIG_PATH: &'static str = ".config/phupload/config.toml";

pub(crate) trait PublisherConfig {}

#[derive(Debug, Deserialize)]
pub(crate) struct FlickrConfig {}

impl PublisherConfig for FlickrConfig {}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
  pub(crate) cloudinary: Option<CloudinaryConfig>,
  pub(crate) script: Option<Vec<ScriptConfig>>,
}

pub(crate) fn read_config() -> Result<Config, UploadError> {
  let home = home_dir();

  if let None = home {
    return Err(UploadError::MissingConfig(
      format!("No config was found at $HOME/{}", CONFIG_PATH).into(),
    ));
  }

  let config_path = home.unwrap().join(Path::new(CONFIG_PATH));
  let raw_config = read_to_string(config_path).map_err(|_| {
    UploadError::MissingConfig(
      format!("Unable to read the config file at $HOME/{}", CONFIG_PATH).into(),
    )
  })?;
  let config: Config = toml::from_str(&raw_config).map_err(|_| {
    UploadError::MalformedConfig(Some(
      "Unable to parse your config file. Please check for any typos".into(),
    ))
  })?;

  Ok(config)
}
