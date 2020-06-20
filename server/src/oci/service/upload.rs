use std::sync::Arc;

use hold::blob::Blob as StorageBlob;

use couchdb::db::Database;

use crate::couchdb::repository::Repository;
use crate::oci::digest::Digest;
use crate::oci::entity::{Repo, Upload, UploadChunk};
use crate::oci::error::{Error, ErrorCode};
use crate::oci::{storage, Result};
use crate::storage::Provider;

pub struct UploadService {
    db: Arc<Database>,
    store: Arc<Provider>,
}

impl UploadService {
    pub fn new(db: Arc<Database>, store: Arc<Provider>) -> Self {
        Self { db, store }
    }

    pub async fn start_upload(&self, repo: &Repo) -> Result<Upload> {
        let upload = Upload::new(repo.group(), repo.name());
        self.save(upload).await.map_err(Error::from)
    }

    pub async fn push_chunk(&self, upload_id: &str, chunk: UploadChunk) -> Result<Upload> {
        let body = chunk.content();
        let mut upload = self
            .find(upload_id)
            .await?
            .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

        upload.add_chunk(chunk);
        let upload = self.save(upload).await?;
        let chunks = upload.chunks();
        let chunk = chunks.last().unwrap();
        let key = chunk.storage_key().unwrap();
        let blob = StorageBlob::new(key.to_string(), body);
        self.store.store_blob(blob).await?;

        Ok(upload)
    }

    pub async fn complete_upload(
        &self,
        upload_id: &str,
        digest: &Digest,
        chunk: Option<UploadChunk>,
    ) -> Result<Upload> {
        log::debug!("Completing upload {} with digest {}", upload_id, digest);

        let upload = self
            .find(upload_id)
            .await?
            .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

        let mut buf = Vec::new();
        let mut chunks = upload.chunks();
        chunks.sort_unstable_by_key(|c| c.start_range());
        for chunk in &chunks {
            let chunk_key = chunk.storage_key().unwrap();
            log::debug!("Fetching chunk {}", chunk_key);
            let blob = self
                .store
                .get_blob(chunk_key)
                .await?
                .ok_or_else(|| Error::from(ErrorCode::BlobUnknown))?;
            let mut content = blob.content().clone();
            buf.append(&mut content);
        }

        let mut chunk = chunk;
        if let Some(chunk) = chunk.as_mut() {
            buf.append(&mut chunk.content());
        }

        let blob_key = storage::blob_key(digest);
        log::debug!("Storing blob {}", blob_key);
        let blob = StorageBlob::new(blob_key, buf);
        self.store.store_blob(blob).await?;
        self.delete(&upload).await?;

        for chunk in &chunks {
            let chunk_key = chunk.storage_key().unwrap();
            self.store.delete_blob(chunk_key).await?;
        }
        Ok(upload)
    }
}

impl Repository<Upload> for UploadService {
    fn db(&self) -> &Database {
        self.db.as_ref()
    }
}
