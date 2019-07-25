use crate::metadata::config::PublisherConfig;
use crate::{PhotoDestination, Upload, UploadError};
use log::{debug, info};
use reqwest::{multipart, Client};
use ring::digest::{digest, SHA1};
use serde::Deserialize;
use std::time::SystemTime;

pub(crate) struct Cloudinary;

#[derive(Debug, Deserialize)]
pub(crate) struct CloudinaryConfig {
  pub(crate) cloud_name: String,
  pub(crate) api_key: String,
  pub(crate) api_secret: String,
}

#[derive(Deserialize, Debug)]
struct UploadResponse {
  public_id: String,
  format: String,
}

impl PublisherConfig for CloudinaryConfig {}

impl PhotoDestination for Cloudinary {
  type Config = CloudinaryConfig;

  fn upload(config: Self::Config, photo: &Upload) -> Result<String, UploadError> {
    info!("Beginning upload to cloudinary...");

    let client = Client::new();
    let timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .expect("System time is invalid")
      .as_secs();
    let upload_tags = photo.metadata.tags.join(",");
    let cloudinary_params = format!(
      "public_id={}&tags={}&timestamp={}{}",
      photo.metadata.title, upload_tags, timestamp, config.api_secret
    );
    let signed_params = digest(&SHA1, cloudinary_params.as_bytes());
    let signed_string = format!("{:?}", signed_params).replace("SHA1:", "");

    match multipart::Form::new()
      .text("api_key", config.api_key)
      .text("timestamp", format!("{}", timestamp))
      .text("public_id", photo.metadata.title.to_owned())
      .text("tags", upload_tags)
      .text("signature", signed_string)
      .file("file", photo.path)
    {
      Ok(post_data) => {
        debug!("Created post data");

        let json = client
          .post(&format!(
            "https://api.cloudinary.com/v1_1/{}/image/upload",
            config.cloud_name
          ))
          .multipart(post_data)
          .send()
          .and_then(|mut cloudinary_res| cloudinary_res.json());

        match json {
          Ok(UploadResponse { public_id, format }) => {
            debug!("Received a response from cloudinary");

            Ok(format!("{}.{}", public_id, format))
          }
          Err(error) => {
            debug!("Received an error from cloudinary {:?}", error);

            Err(UploadError::BadGateway(Some(format!("{:?}", error))))
          }
        }
      }
      Err(error) => {
        debug!("Couldn't create POST data {:?}", error);

        Err(UploadError::BadGateway(Some(format!("{:?}", error))))
      }
    }
  }
}
