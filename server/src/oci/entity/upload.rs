use actix_web::http::HeaderMap;
use bytes::Bytes;
use http::header;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use enseada::guid::Guid;

use crate::couchdb::repository::Entity;
use crate::oci::error::{Error, ErrorCode};
use crate::oci::storage;
use crate::oci::Result;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Upload {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    image: String,
    chunks: Vec<UploadChunk>,
    latest_offset: usize,
}

impl Upload {
    pub fn new(group: &str, name: &str) -> Self {
        Self {
            image: format!("{}/{}", group, name),
            ..Default::default()
        }
    }

    pub fn chunks(&self) -> Vec<&UploadChunk> {
        self.chunks.iter().collect()
    }

    pub fn add_chunk(&mut self, chunk: UploadChunk) -> &mut Self {
        let mut chunk = chunk;
        let key = storage::chunk_key(self.id.id(), chunk.start_range);
        chunk.storage_key = Some(key);
        self.latest_offset = chunk.end_range;
        self.chunks.push(chunk);
        self
    }

    pub fn latest_offset(&self) -> usize {
        self.latest_offset
    }
}

impl Default for Upload {
    fn default() -> Self {
        Self {
            id: Self::build_guid(&Uuid::new_v4().to_string()),
            rev: None,
            image: String::new(),
            chunks: Vec::new(),
            latest_offset: 0,
        }
    }
}

impl Entity for Upload {
    fn build_guid(id: &str) -> Guid {
        Guid::partitioned("oci_upload", id)
    }

    fn id(&self) -> &Guid {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UploadChunk {
    start_range: usize,
    end_range: usize,
    storage_key: Option<String>,
    #[serde(skip)]
    content: Vec<u8>,
}

impl UploadChunk {
    pub fn start_range(&self) -> usize {
        self.start_range
    }

    pub fn end_range(&self) -> usize {
        self.end_range
    }

    pub fn storage_key(&self) -> Option<&str> {
        self.storage_key.as_deref()
    }

    pub fn set_storage_key(&mut self, key: String) -> &mut Self {
        self.storage_key = Some(key);
        self
    }

    pub fn content(&self) -> Vec<u8> {
        self.content.clone()
    }

    pub fn from_request(headers: &HeaderMap, body: Bytes) -> Result<Self> {
        let content_length = headers
            .get(header::CONTENT_LENGTH)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.parse().ok())
            .unwrap_or_else(|| body.len());

        let content_range = headers.get(header::CONTENT_RANGE);
        let (start, end) = if let Some(hdr) = content_range {
            let value = hdr
                .to_str()
                .map_err(|_| Error::from(ErrorCode::Unsupported))?;
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
