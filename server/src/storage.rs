use hold_s3::{S3Config, S3Provider};

use enseada::error::Error;

use crate::config::{Storage, CONFIG};

pub type Provider = Box<dyn hold::provider::Provider + Send + Sync>;

pub fn new_provider() -> Result<Provider, Error> {
    match CONFIG.storage() {
        Storage::S3 { bucket, endpoint } => {
            let provider = new_s3_provider(bucket.clone(), endpoint.clone());
            Ok(Box::new(provider))
        }
        Storage::Unknown => Err(unknown_provider_error()),
    }
}

pub fn new_s3_provider(bucket: String, endpoint: Option<String>) -> S3Provider {
    S3Provider::new_with_config(S3Config {
        bucket,
        endpoint,
        region: None,
        credentials: None,
    })
}

pub fn unknown_provider_error() -> Error {
    Error::new("unknown storage provider")
}
