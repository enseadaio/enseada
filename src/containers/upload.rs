use std::sync::Arc;

use actix_web::{HttpMessage, HttpRequest};
use actix_web::http::HeaderMap;
use actix_web::web::{Bytes, ServiceConfig};
use futures::StreamExt;
use hold::blob::Blob;
use http::header;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::CONFIG;
use crate::containers::{Result, storage};
use crate::containers::digest::Digest;
use crate::containers::error::{Error, ErrorCode};
use crate::containers::name::Name;
use crate::containers::storage::Provider;
use crate::couchdb;
use crate::couchdb::db::{self, Database};
use crate::guid::Guid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Upload {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    image: Name,
    chunks: Vec<UploadChunk>,
    latest_offset: usize,
}

impl Upload {
    pub fn build_guid(uuid: &str) -> Guid {
        Guid::partitioned("upload", &uuid)
    }

    pub fn start(image: Name) -> Self {
        let id = Self::build_guid(&Uuid::new_v4().to_string());
        Upload {
            id,
            rev: None,
            image,
            chunks: Vec::new(),
            latest_offset: 0,
        }
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn latest_offset(&self) -> usize {
        self.latest_offset
    }

    pub fn add_chunk(&mut self, chunk: UploadChunk) -> &mut Self {
        let mut chunk = chunk;
        let key = storage::chunk_key(self.id.id(), chunk.start_range);
        chunk.storage_key = Some(key);
        self.latest_offset = chunk.end_range;
        self.chunks.push(chunk);
        self
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadChunk {
    start_range: usize,
    end_range: usize,
    storage_key: Option<String>,
    #[serde(skip)]
    content: Vec<u8>,
}

impl UploadChunk {
    pub fn storage_key(&self) -> Option<&String> {
        self.storage_key.as_ref()
    }

    pub fn from_request(headers: &HeaderMap, body: Bytes) -> Result<Self> {
        let content_length = headers.get(header::CONTENT_LENGTH)
            .ok_or_else(|| Error::new(ErrorCode::Unsupported, "Content-Length header is missing"))?;

        let content_length = content_length.to_str().map_err(|_| Error::from(ErrorCode::Unsupported))?;
        let content_length: usize = content_length.parse().unwrap();

        let content_range = headers.get(header::CONTENT_RANGE);
        let (start, end) = if let Some(hdr) = content_range {
            let value = hdr.to_str().map_err(|_| Error::from(ErrorCode::Unsupported))?;
            let range: Vec<&str> = value.split('-').collect();
            let start: usize = range.first().unwrap().parse().unwrap();
            let end: usize = range.last().unwrap().parse().unwrap();
            (start, end)
        } else {
            (0, content_length)
        };

        Ok(UploadChunk {
            start_range: start,
            end_range: end,
            storage_key: None,
            content: body.to_vec(),
        })
    }
}

pub struct UploadService {
    db: Database,
    store: Arc<Provider>,
}

impl UploadService {
    pub fn new(db: Database, store: Arc<Provider>) -> Self {
        UploadService { db, store }
    }

    pub async fn start_upload(&self, image: Name) -> Result<Upload> {
        let upload = Upload::start(image);
        self.save_upload(upload).await
    }

    pub async fn save_upload(&self, upload: Upload) -> Result<Upload> {
        let id = upload.id.to_string();
        let res = self.db.put(&id, &upload).await?;
        let mut upload = upload;
        upload.rev = Some(res.rev);
        Ok(upload)
    }

    pub async fn find_upload(&self, upload_id: &str) -> Result<Option<Upload>> {
        let id = Upload::build_guid(upload_id).to_string();
        self.db.get(&id).await.map_err(Error::from)
    }

    pub async fn delete_upload(&self, upload: Upload) -> Result<()> {
        let upload = if upload.rev.is_some() {
            upload
        } else {
            self.db.get(&upload.id.to_string()).await?
                .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?
        };

        self.db.delete(&upload.id.to_string(), upload.rev.as_ref().unwrap()).await
            .map_err(Error::from)
    }

    pub async fn push_chunk(&self, upload_id: &str, chunk: UploadChunk) -> Result<Upload> {
        let mut chunk = chunk;
        let body = chunk.content.drain(..).collect();
        let mut upload = self.find_upload(upload_id).await?
            .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

        upload.add_chunk(chunk);
        let upload = self.save_upload(upload).await?;
        let chunk = upload.chunks.last().unwrap();
        let key = chunk.storage_key().unwrap();
        let blob = Blob::new(key.clone(), body);
        self.store.store_blob(blob).await?;

        Ok(upload)
    }

    pub async fn complete_upload(&self, upload_id: &str, digest: &Digest, chunk: UploadChunk) -> Result<()> {
        log::debug!("Completing upload {} with digest {}", upload_id, digest);

        let mut upload = self.find_upload(upload_id).await?
            .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

        let mut chunk = chunk;
        let key = storage::chunk_key(upload.id.id(), chunk.start_range);
        chunk.storage_key = Some(key);

        let mut buf = Vec::new();
        upload.chunks.sort_unstable_by_key(|c| c.start_range);
        for chunk in &upload.chunks {
            let chunk_key = chunk.storage_key().unwrap();
            log::debug!("Fetching chunk {}", chunk_key);
            let blob = self.store.get_blob(chunk_key).await?
                .ok_or_else(|| Error::from(ErrorCode::BlobUnknown))?;
            let mut content = blob.content().clone();
            buf.append(&mut content);
            self.store.delete_blob(&chunk_key).await?;
        }
        buf.append(&mut chunk.content);

        let blob_key = storage::blob_key(digest);
        log::debug!("Storing blob {}", blob_key);
        let blob = Blob::new(blob_key, buf);
        self.store.store_blob(blob).await?;
        self.delete_upload(upload).await
    }
}

pub fn add_upload_service(app: &mut ServiceConfig) {
    let couch = &couchdb::SINGLETON;
    let db = couch.database(db::name::OCI, true);
    let provider = storage::new_provider(&CONFIG).expect("docker storage provider");
    let service = UploadService::new(db, Arc::new(provider));
    app.data(service);
}