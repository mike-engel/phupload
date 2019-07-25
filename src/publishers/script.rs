use crate::metadata::config::PublisherConfig;
use crate::{PhotoDestination, Upload, UploadError};
use log::{debug, info};
use serde::Deserialize;
use serde_json::{json, to_string};
use std::path::Path;
use std::process::Command;

pub(crate) struct Script;

#[derive(Debug, Deserialize)]
pub(crate) struct ScriptConfig {
  pub(crate) path: String,
}

impl PublisherConfig for ScriptConfig {}

impl PhotoDestination for Script {
  type Config = ScriptConfig;

  fn upload(config: Self::Config, photo: &Upload) -> Result<String, UploadError> {
    info!("Beginning custom script...");

    let data = json!({
      "url": photo.url.clone().unwrap_or("".into()),
      "name": photo.metadata.title,
      "description": photo.metadata.description,
      "heightAt1200": photo.metadata.height_at_1200,
      "camera": photo.metadata.camera,
      "focalLength": photo.metadata.focal_length,
      "iso": photo.metadata.iso,
      "aperture": photo.metadata.aperture,
      "shutterSpeed": photo.metadata.shutter_speed,
      "createdAt": photo.metadata.created_at,
      "tags": photo.metadata.tags
    });

    dbg!(&config.path);

    let result = Command::new(&config.path)
      .arg(to_string(&data).unwrap())
      .current_dir(Path::new(&config.path).parent().unwrap())
      .output()
      .map_err(|err| {
        debug!("Error execuring a custom script: {:?}", err);

        UploadError::UnknownError(Some(
          "Error executing a custom script. Is the path correct?".into(),
        ))
      })?;

    match result.status.code() {
      Some(0) => {
        debug!(
          "Successfully executed a script: {:?}",
          String::from_utf8(result.stdout)
        );

        Ok("".into())
      }
      _ => {
        let output = String::from_utf8(result.stderr);
        debug!("Error executing custom script: {:?}", output);

        Err(UploadError::UnknownError(Some(
          format!("Error executing a custom script: {:?}", output).into(),
        )))
      }
    }
  }
}
