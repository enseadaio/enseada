use crate::config::{Configuration, Storage};
use crate::containers::digest::Digest;
use crate::error::Error;
use crate::storage;

pub type Provider = Box<dyn hold::provider::Provider>;

pub fn new_provider(cfg: &Configuration) -> Result<Provider, Error> {
    match cfg.storage() {
        Storage::S3 { bucket, endpoint } => {
            let provider = storage::new_s3_provider(bucket.clone(), endpoint.clone());
            Ok(Box::new(provider))
        }
        Storage::Unknown => Err(storage::unknown_provider_error())
    }
}

pub fn chunk_key(upload_id: &str, chunk_offset: usize) -> String {
    format!("artifacts/oci/uploads/{}/chunks/{}", upload_id, chunk_offset)
}

pub fn blob_key(digest: &Digest) -> String {
    format!("artifacts/oci/blobs/{}", digest.to_string())
}