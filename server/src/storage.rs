use hold_s3::{S3Config, S3Credentials, S3Provider};

use enseada::error::Error;
use enseada::storage::Provider;

use crate::config::{Configuration, Storage};

pub fn new_provider(cfg: &Configuration) -> Result<Provider, Error> {
    match cfg.storage() {
        Storage::S3 {
            bucket,
            endpoint,
            access_key_id,
            secret_access_key,
        } => {
            let provider = new_s3_provider(
                bucket.clone(),
                endpoint.clone(),
                access_key_id.clone(),
                secret_access_key.clone(),
            );
            Ok(Box::new(provider))
        }
        Storage::Unknown => Err(unknown_provider_error()),
    }
}

pub fn new_s3_provider(
    bucket: String,
    endpoint: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
) -> S3Provider {
    let credentials = if let (Some(access_key_id), Some(secret_access_key)) =
        (access_key_id, secret_access_key)
    {
        Some(S3Credentials {
            access_key_id,
            secret_access_key,
        })
    } else {
        None
    };

    S3Provider::new_with_config(S3Config {
        bucket,
        endpoint,
        region: None,
        credentials,
    })
}

pub fn unknown_provider_error() -> Error {
    Error::new("unknown storage provider")
}
