mod metadata;
mod publishers;

use crate::metadata::config::read_config;
use crate::metadata::exif::{get_metadata, Metadata};
use crate::publishers::cloudinary::Cloudinary;
use crate::publishers::flickr::Flickr;
use crate::publishers::script::Script;
use clap::{App, Arg, ArgMatches};
use metadata::config::PublisherConfig;
use simplelog::{LevelFilter, TermLogger};
use log::debug;

#[derive(Debug)]
pub(crate) enum UploadError {
	BadGateway(Option<String>),
	MalformedConfig(Option<String>),
	MissingConfig(Option<String>),
	UnknownError(Option<String>),
}

pub(crate) trait PhotoDestination {
	type Config: PublisherConfig;

	fn upload(config: Self::Config, photo: &Upload) -> Result<String, UploadError>;
}

#[derive(Clone, Debug)]
pub struct Upload<'a> {
	path: &'a str,
	photo: &'a [u8],
	metadata: Metadata,
	url: Option<String>,
}

fn get_matches<'a>() -> ArgMatches<'a> {
	App::new("photo uploader")
		.version("0.1")
		.about("Publish a new photo to several platforms and websites")
		.arg(
			Arg::with_name("DEBUG")
				.short("-d")
				.long("--debug")
				.help("Print extra information to the console")
				.takes_value(false),
		)
		.arg(
			Arg::with_name("PATH")
				.help("The path to the photo to upload")
				.required(true)
				.index(1),
		)
		.get_matches()
}

fn main() -> Result<(), UploadError> {
	let matches = get_matches();
	let log_level = if matches.is_present("DEBUG") {
		LevelFilter::Debug
	} else {
		LevelFilter::Info
	};

	match TermLogger::init(log_level, simplelog::Config::default()) {
		Ok(_) => {}
		Err(error) => panic!("Could not set up TermLogger {:?}", error),
	};

	let config = read_config()?;
	let photo_path = matches.value_of("PATH").unwrap();
	let metadata = get_metadata(photo_path)?;

	debug!("metadata: {:?}", metadata);

	let mut photo_to_upload = Upload {
		path: photo_path,
		photo: &[],
		metadata: metadata,
		url: None,
	};

	let url = match config.cloudinary {
		Some(cloudinary_config) => Some(Cloudinary::upload(cloudinary_config, &photo_to_upload)?),
		None => None,
	};

	photo_to_upload.url = url;

	if let Some(scripts) = config.script {
		for script in scripts {
			Script::upload(script, &photo_to_upload)?;
		}
	}

	if let Some(flickr_config) = config.flickr {
		Flickr::upload(flickr_config, &photo_to_upload)?;
	}

	Ok(())
}
