use crate::metadata::config::{read_config, write_config, Config, PublisherConfig};
use crate::{PhotoDestination, Upload, UploadError};
use base64;
use log::{debug, info};
use rand::{thread_rng, Rng};
use reqwest::{multipart, Client};
use ring::hmac::{sign, Key, HMAC_SHA1_FOR_LEGACY_USE_ONLY};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{Duration, SystemTime};
use url::Url;
use urlencoding::encode;

const FLICKR_API_URL: &'static str = "https://www.flickr.com/services";
const FLICKR_UPLOAD_URL: &'static str = "https://up.flickr.com/services/upload/";
const FLICKR_CALLBACK_URL: &'static str = "http://localhost:8282";

pub(crate) struct Flickr;

struct Oauth;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct FlickrConfig {
	pub(crate) oauth_client_key: String,
	pub(crate) oauth_client_secret: String,
	pub(crate) oauth_token: Option<String>,
	pub(crate) oauth_token_secret: Option<String>,
	pub(crate) oauth_verifier: Option<String>,
	pub(crate) oauth_access_token: Option<String>,
	pub(crate) oauth_access_token_secret: Option<String>,
}

impl PublisherConfig for FlickrConfig {}

impl Oauth {
	fn nonce() -> String {
		thread_rng()
			.sample_iter(rand::distributions::Alphanumeric)
			.take(8)
			.collect()
	}

	fn timestamp() -> String {
		SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("System time is invalid")
			.as_secs()
			.to_string()
	}

	fn key(config: &FlickrConfig, token: Option<&str>) -> Key {
		Key::new(
			HMAC_SHA1_FOR_LEGACY_USE_ONLY,
			format!(
				"{}&{}",
				encode(config.oauth_client_secret.as_str()),
				token.unwrap_or("")
			)
			.as_bytes(),
		)
	}

	fn create_signature(
		key: &Key,
		method: &str,
		path: String,
		params: &mut Vec<(&str, &str)>,
	) -> String {
		params.sort_by(|a, b| a.0.cmp(&b.0));

		let param_queries: Vec<String> = params
			.into_iter()
			.map(|(k, v)| format!("{}={}", k, encode(v)))
			.collect();
		let param_query = param_queries.join("&");
		let to_sign = format!(
			"{}&{}&{}",
			method,
			encode(&path),
			encode(param_query.as_str())
		);

		Oauth::sign(key, to_sign.as_bytes())
	}

	fn sign(key: &Key, data: &[u8]) -> String {
		base64::encode(&sign(key, data))
	}

	fn get_request_token(config: FlickrConfig) -> Result<FlickrConfig, UploadError> {
		let client = Client::new();
		let timestamp = Oauth::timestamp();
		let nonce = Oauth::nonce();
		let key = Oauth::key(&config, None);
		let mut params = vec![
			("oauth_nonce", nonce.as_str()),
			("oauth_timestamp", timestamp.as_str()),
			("oauth_consumer_key", config.oauth_client_key.as_str()),
			("oauth_version", "1.0"),
			("oauth_signature_method", "HMAC-SHA1"),
			("oauth_callback", FLICKR_CALLBACK_URL),
		];
		let signature = Oauth::create_signature(
			&key,
			"GET",
			format!("{}{}", FLICKR_API_URL, "/oauth/request_token"),
			&mut params,
		);

		params.extend(&[("oauth_signature", signature.as_str())]);

		let result = client
			.get(&format!("{}/oauth/request_token", FLICKR_API_URL))
			.query(&params)
			.send()
			.and_then(|mut res| res.text())
			.map_err(|err| {
				debug!("Error getting flickr request token: {:?}", err);

				UploadError::BadGateway(Some("Error getting flickr request token".into()))
			})?;
		let response_params: Vec<(&str, &str)> = result
			.split("&")
			.collect::<Vec<&str>>()
			.into_iter()
			.map(|a| {
				let split: Vec<&str> = a.split("=").collect();

				(split[0], split[1])
			})
			.collect();
		let mut hash: HashMap<String, String> = HashMap::new();

		for (k, v) in response_params {
			hash.insert(k.into(), v.into());
		}

		Ok(FlickrConfig {
			oauth_token: hash.get("oauth_token").map(|v| v.to_owned()),
			oauth_token_secret: hash.get("oauth_token_secret").map(|v| v.to_owned()),
			..config
		})
	}

	fn authorize_app(config: FlickrConfig) -> Result<FlickrConfig, UploadError> {
		let addr = "0.0.0.0:8282";
		let mut oauth_verifier = String::new();
		let listener = TcpListener::bind(addr).unwrap();

		info!(
			"Please open {}/oauth/authorize?oauth_token={}&perms=write",
			FLICKR_API_URL,
			config.oauth_token.to_owned().unwrap()
		);

		for stream in listener.incoming() {
			match stream {
				Ok(mut stream) => {
					{
						let mut reader = BufReader::new(&stream);
						let mut request_line = String::new();

						reader.read_line(&mut request_line).unwrap();

						let redirect_url = request_line.split_whitespace().nth(1).unwrap();
						let full_url = format!("{}{}", FLICKR_CALLBACK_URL, redirect_url);
						let url = Url::parse(&full_url).unwrap();
						let (_, verifier) = url
							.query_pairs()
							.find(|(k, _)| k == "oauth_verifier")
							.unwrap();

						oauth_verifier = verifier.into();
					}

					let message = "Go back to your terminal";
					let response = format!(
						"HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
						message.len(),
						message
					);

					stream.write_all(response.as_bytes()).unwrap();

					break;
				}
				Err(err) => {
					debug!("Error starting flickr auth server: {:?}", err);

					return Err(UploadError::BadGateway(Some(
						"Error starting flickr auth server. Please try again.".into(),
					)));
				}
			}
		}

		Ok(FlickrConfig {
			oauth_verifier: Some(oauth_verifier),
			..config
		})
	}

	fn get_access_token(config: FlickrConfig) -> Result<FlickrConfig, UploadError> {
		let mut hash: HashMap<String, String> = HashMap::new();
		let client = Client::new();
		let timestamp = Oauth::timestamp();
		let nonce = Oauth::nonce();
		let secret = &config.oauth_token_secret.clone().unwrap();
		let key = Oauth::key(&config, Some(&secret.as_str()));
		let verifier = &config.oauth_verifier.clone().unwrap();
		let token = &config.oauth_token.clone().unwrap();
		let mut params = vec![
			("oauth_nonce", nonce.as_str()),
			("oauth_timestamp", timestamp.as_str()),
			("oauth_consumer_key", &config.oauth_client_key.as_str()),
			("oauth_version", "1.0"),
			("oauth_signature_method", "HMAC-SHA1"),
			("oauth_callback", FLICKR_CALLBACK_URL),
			("oauth_verifier", verifier.as_str()),
			("oauth_token", token.as_str()),
		];
		let signature = Oauth::create_signature(
			&key,
			"GET",
			format!("{}{}", FLICKR_API_URL, "/oauth/access_token"),
			&mut params,
		);

		params.extend(&[("oauth_signature", signature.as_str())]);

		let result = client
			.get(&format!("{}/oauth/access_token", FLICKR_API_URL))
			.query(&params)
			.send()
			.and_then(|mut res| res.text())
			.map_err(|err| {
				debug!("Error getting flickr access token: {:?}", err);

				UploadError::BadGateway(Some("Error getting flickr access token".into()))
			})?;
		let response_params: Vec<(&str, &str)> = result
			.split("&")
			.collect::<Vec<&str>>()
			.into_iter()
			.map(|a| {
				let split: Vec<&str> = a.split("=").collect();

				(split[0], split[1])
			})
			.collect();

		for (k, v) in response_params {
			hash.insert(k.into(), v.into());
		}

		Ok(FlickrConfig {
			oauth_access_token: hash.get("oauth_token").map(|v| v.to_owned()),
			oauth_access_token_secret: hash.get("oauth_token_secret").map(|v| v.to_owned()),
			..config
		})
	}
}

impl PhotoDestination for Flickr {
	type Config = FlickrConfig;

	fn upload(config: Self::Config, photo: &Upload) -> Result<String, UploadError> {
		info!("Beginning upload to Flickr...");

		let auth_config = match config.oauth_access_token {
			Some(_) => config,
			None => {
				let token_config = Oauth::get_request_token(config)?;
				let verifier_config = Oauth::authorize_app(token_config)?;
				let access_config = Oauth::get_access_token(verifier_config)?;
				let final_config = Self::save_access_token(access_config)?;

				final_config
			}
		};
		let client = Client::builder()
			.timeout(Duration::from_secs(120))
			.build()
			.unwrap();
		let timestamp = Oauth::timestamp();
		let nonce = Oauth::nonce();
		let key = Oauth::key(
			&auth_config,
			Some(
				&auth_config
					.oauth_access_token_secret
					.to_owned()
					.unwrap()
					.as_str(),
			),
		);
		let token = auth_config.oauth_access_token.clone().unwrap();
		let tags = photo.metadata.tags.join(" ");
		let title = &photo.metadata.title;
		let description = &photo.metadata.description;
		let mut params = vec![
			("oauth_nonce", nonce.as_str()),
			("oauth_timestamp", timestamp.as_str()),
			("oauth_consumer_key", auth_config.oauth_client_key.as_str()),
			("oauth_version", "1.0"),
			("oauth_signature_method", "HMAC-SHA1"),
			("oauth_token", token.as_str()),
			("format", "json"),
			("title", title.as_str()),
			("description", description.as_str()),
			("tags", tags.as_str()),
			("content_type", "1"),
		];
		let signature = Oauth::create_signature(&key, "POST", FLICKR_UPLOAD_URL.into(), &mut params);

		params.extend(&[("oauth_signature", signature.as_str())]);

		let mut body = multipart::Form::new();

		for (k, v) in params {
			body = body.text(k, String::from(v));
		}

		client
			.post(FLICKR_UPLOAD_URL)
			.multipart(body.file("photo", photo.path).unwrap())
			.send()
			.and_then(|mut res| res.text())
			.map_err(|err| {
				debug!("Error publishing the photo to Flickr: {:?}", err);

				UploadError::BadGateway(Some("Error publishing the photo to Flickr".into()))
			})?;

		Ok("".into())
	}
}

impl Flickr {
	fn save_access_token(config: FlickrConfig) -> Result<FlickrConfig, UploadError> {
		let existing_config = read_config()?;
		let new_config = Config {
			flickr: Some(config.clone()),
			..existing_config
		};

		write_config(new_config)?;

		Ok(config)
	}
}
