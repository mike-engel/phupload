use crate::metadata::config::FlickrConfig;
use crate::{PhotoDestination, Upload, UploadError};
// use reqwest::Client;

pub(crate) struct Flickr;

impl PhotoDestination for Flickr {
  type Config = FlickrConfig;

  fn upload(_: Self::Config, __: &Upload) -> Result<String, UploadError> {
    Ok(String::from(""))
  }
}
