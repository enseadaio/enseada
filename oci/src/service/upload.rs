use std::sync::Arc;

use async_trait::async_trait;
use futures::{stream, Stream, StreamExt};

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::{Entity, Repository};
use enseada::storage::blob::Blob as StorageBlob;
use enseada::storage::{ByteChunk, Provider};
use events::EventHandler;

use crate::digest::Digest;
use crate::entity::{Repo, Upload, UploadChunk};
use crate::error::{Error, ErrorCode};
use crate::events::RepoDeleted;
use crate::{storage, Result};
use bytes::Bytes;

#[derive(Debug)]
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

    // pub async fn push_chunk<B: Stream<Item = ByteChunk> + Send + Sync + 'static>(
    pub async fn push_chunk(
        &self,
        mut upload: Upload,
        chunk: UploadChunk,
        body: Bytes,
    ) -> Result<Upload> {
        if chunk.start_range() != upload.latest_offset() {
            log::debug!(
                "chunk range {} is not compatible with current offset {}",
                chunk.start_range(),
                upload.latest_offset()
            );
            return Err(Error::from(ErrorCode::RequestedRangeNotSatisfiable(
                upload.latest_offset(),
            )));
        }

        upload.add_chunk(chunk);
        let chunks = upload.chunks();
        let chunk = chunks.last().unwrap();
        let key = chunk.storage_key().unwrap();

        let blob = StorageBlob::new(
            key.to_string(),
            chunk.size(),
            stream::once(async move { Ok(body) }),
        );
        log::debug!("storing blob {}", key);
        self.store.store_blob(blob).await?;
        log::debug!("blob stored");

        let upload = self.save(upload).await?;
        Ok(upload)
    }

    // pub async fn complete_upload<B: Stream<Item = ByteChunk> + Send + Sync + 'static>(
    pub async fn complete_upload(
        &self,
        mut upload: Upload,
        digest: &Digest,
        chunk: Option<(UploadChunk, Bytes)>,
    ) -> Result<Upload> {
        log::debug!(
            "completing upload {} with digest {}",
            upload.id().id(),
            digest
        );

        let upload_latest_offset = upload.latest_offset();

        let mut blobs = Vec::new();
        let mut chunks = upload.chunks_mut();
        let storage_keys: Vec<String> = chunks
            .iter()
            .map(|chunk| chunk.storage_key().unwrap().to_string())
            .collect();
        chunks.sort_unstable_by_key(|c| c.start_range());
        for chunk in &chunks {
            let chunk_key = chunk.storage_key().unwrap();
            log::debug!("fetching chunk {}", chunk_key);
            let blob = self
                .store
                .get_blob(chunk_key)
                .await?
                .ok_or_else(|| Error::from(ErrorCode::BlobUnknown))?;
            blobs.push(blob);
        }

        let latest_offset = chunk
            .as_ref()
            .map(|(chunk, _)| chunk)
            .map(UploadChunk::end_range)
            .unwrap_or(upload_latest_offset);

        if let Some((chunk, body)) = chunk {
            // we don't care about the storage key because this blob will never be stored
            // see stream mapping a few line below
            let blob = StorageBlob::new(
                "we-dont-care",
                chunk.size(),
                stream::once(async move { Ok(body) }),
            );
            blobs.push(blob);
        }

        let buf = stream::iter(blobs.into_iter())
            .map(StorageBlob::into_byte_stream)
            .flatten();

        let blob_key = storage::blob_key(digest);
        log::debug!("storing blob {}", blob_key);
        let blob = StorageBlob::new(blob_key, latest_offset, buf);
        self.store.store_blob(blob).await?;
        log::debug!("blob stored");
        log::debug!("deleting upload");
        self.delete(&upload).await?;
        log::debug!("upload deleted");

        for chunk_key in storage_keys {
            self.store.delete_blob(&chunk_key).await?;
        }
        Ok(upload)
    }
}

impl Repository<Upload> for UploadService {
    fn db(&self) -> &Database {
        self.db.as_ref()
    }
}

#[async_trait]
impl EventHandler<RepoDeleted> for UploadService {
    async fn handle(&self, event: &RepoDeleted) {
        let image = format!("{}/{}", &event.group, &event.name);
        match self
            .find_all(
                100,
                0,
                serde_json::json!({
                  "image": image,
                }),
            )
            .await
        {
            Ok(page) => {}
            Err(err) => {
                log::error!("{}", err);
            }
        }
    }
}
