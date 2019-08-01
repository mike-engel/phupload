use crate::publishers::cloudinary::CloudinaryConfig;
use crate::publishers::flickr::FlickrConfig;
use crate::publishers::script::ScriptConfig;
use crate::UploadError;
use dirs::home_dir;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::path::Path;

const CONFIG_PATH: &'static str = ".config/phupload/config.toml";

pub(crate) trait PublisherConfig {}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
	pub(crate) cloudinary: Option<CloudinaryConfig>,
	pub(crate) script: Option<Vec<ScriptConfig>>,
	pub(crate) flickr: Option<FlickrConfig>,
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

pub(crate) fn write_config(config: Config) -> Result<(), UploadError> {
	let home = home_dir();

	if let None = home {
		return Err(UploadError::MissingConfig(
			format!("No config was found at $HOME/{}", CONFIG_PATH).into(),
		));
	}

	let config_path = home.unwrap().join(Path::new(CONFIG_PATH));
	let toml_config = toml::to_string(&config).map_err(|err| {
		debug!("Error serializing the new config: {:?}", err);

		UploadError::MalformedConfig(Some("Error serializing the new config".into()))
	})?;

	match write(config_path, toml_config) {
		Ok(_) => Ok(()),
		Err(err) => {
			debug!("Error saving updated config: {:?}", err);

			Err(UploadError::UnknownError(Some(
				"Error saving the updated config. Please try again.".into(),
			)))
		}
	}
}
