use crate::UploadError;
use heck::TitleCase;
use log::debug;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Metadata {
	pub(crate) camera: String,
	pub(crate) focal_length: String,
	pub(crate) iso: String,
	pub(crate) aperture: String,
	pub(crate) shutter_speed: String,
	pub(crate) title: String,
	pub(crate) description: String,
	pub(crate) tags: Vec<String>,
	pub(crate) height_at_1200: i32,
	pub(crate) created_at: String,
}

// Convert internal model names to friendly ones
// NOT COMPREHENSIVE
fn convert_model(model: &str) -> String {
	match model {
		"ILCE-7RM2" => "a7r II".into(),
		"ILCE-7RM3" => "a7r III".into(),
		"ILCE-7RM4" => "a7r IV".into(),
		"ILCE-7SM2" => "a7s II".into(),
		"ILCE-7SM3" => "a7s III".into(),
		"ILCE-7SM4" => "a7s IV".into(),
		_ => model.into(),
	}
}

impl Metadata {
	fn from_exiftool(output: Vec<u8>) -> Metadata {
		let mut data: HashMap<String, String> = HashMap::new();
		let str_output = String::from_utf8(output).unwrap();
		let lines = str_output.split("\n");
		let regex = Regex::new("^(?P<k>.+): (?P<v>.+)$").unwrap();

		for line in lines {
			if let "" = line {
				continue;
			}

			let matches = regex.captures(&line).unwrap();

			match (matches.name("k"), matches.name("v")) {
				(Some(key), Some(value)) => {
					data.insert(key.as_str().into(), value.as_str().into());
				}
				_ => {}
			};
		}

		Metadata {
			camera: format!(
				"{} {}",
				data
					.get("Make")
					.unwrap_or(&String::from(""))
					.to_title_case(),
				convert_model(data.get("Model").unwrap_or(&String::from(""))),
			),
			focal_length: data.get("FocalLength").unwrap_or(&String::from("")).into(),
			iso: data.get("ISO").unwrap_or(&String::from("")).into(),
			aperture: data
				.get("ApertureValue")
				.unwrap_or(&String::from("0.0"))
				.into(),
			shutter_speed: data
				.get("ShutterSpeedValue")
				.unwrap_or(&String::from(""))
				.into(),
			title: data
				.get("Title")
				.unwrap_or(&String::from(""))
				.to_title_case(),
			description: data.get("Description").unwrap_or(&String::from("")).into(),
			created_at: data.get("Description").unwrap_or(&String::from("")).into(),
			tags: {
				let tags_str = data.get("Keywords").unwrap_or(&String::from("")).to_owned();
				let tag_list: Vec<&str> = tags_str.split(", ").collect();

				[tag_list, vec!["upload"]]
					.concat()
					.into_iter()
					.map(|s| String::from(s))
					.filter(|s| s.len() > 0)
					.collect()
			},
			height_at_1200: {
				let width = data
					.get("ImageWidth")
					.unwrap_or(&String::from("1"))
					.parse::<i32>()
					.unwrap_or(1);
				let height = data
					.get("ImageHeight")
					.unwrap_or(&String::from("1"))
					.parse::<i32>()
					.unwrap_or(1);

				height * 1200 / width
			},
		}
	}
}

pub(crate) fn get_metadata(path: &str) -> Result<Metadata, UploadError> {
	let exif_return = Command::new("exiftool")
		.args(&[
			"-S",
			"-EXIF:ISO",
			"-EXIF:ShutterSpeedValue",
			"-EXIF:ApertureValue",
			"-EXIF:FocalLength",
			"-EXIF:Make",
			"-EXIF:Model",
			"-ImageWidth",
			"-ImageHeight",
			"-Title",
			"-Keywords",
			"-Description",
			"-DateTimeCreated",
			"-d %Y-%m-%dT%H:%M:%S%z",
			path,
		])
		.output()
		.map_err(|err| {
			debug!("Error gathering EXIF data: {:?}", err);

			UploadError::UnknownError(Some(
				"Error gathering EXIF data. Is exiftool installed?".into(),
			))
		})?;

	Ok(Metadata::from_exiftool(exif_return.stdout))
}
