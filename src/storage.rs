use hold_s3::{S3Config, S3Provider};
use http::StatusCode;

use crate::error::Error;

pub fn new_s3_provider(bucket: String, endpoint: Option<String>) -> S3Provider {
    S3Provider::new_with_config(S3Config {
        bucket,
        endpoint,
        region: None,
        credentials: None,
    })
}

pub fn unknown_provider_error() -> Error {
    Error::new("unknown storage provider", Some(StatusCode::INTERNAL_SERVER_ERROR))
}