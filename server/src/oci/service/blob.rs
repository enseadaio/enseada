use std::sync::Arc;

use couchdb::db::Database;

use crate::couchdb::repository::Repository;
use crate::oci::digest::Digest;
use crate::oci::entity::Blob;
use crate::oci::error::{Error, ErrorCode};
use crate::oci::{storage, Result};
use crate::storage::Provider;

#[derive(Debug)]
pub struct BlobService {
    db: Arc<Database>,
    store: Arc<Provider>,
}

impl BlobService {
    pub fn new(db: Arc<Database>, store: Arc<Provider>) -> Self {
        Self { db, store }
    }

    pub async fn fetch_content(&self, digest: &Digest) -> Result<Vec<u8>> {
        let storage_key = storage::blob_key(digest);
        let blob = self.store.get_blob(&storage_key).await?;
        match blob {
            Some(blob) => Ok(blob.content().clone()),
            None => Err(Error::from(ErrorCode::BlobUnknown)),
        }
    }

    pub async fn delete_blob(&self, blob: &Blob) -> Result<()> {
        let storage_key = storage::blob_key(blob.digest());
        self.delete(blob).await?;
        self.store.delete_blob(&storage_key).await?;
        Ok(())
    }
}

impl Repository<Blob> for BlobService {
    fn db(&self) -> &Database {
        self.db.as_ref()
    }
}
