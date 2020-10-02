use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;

use crate::storage;

#[derive(Debug, Deserialize, Serialize)]
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

    pub fn chunks_mut(&mut self) -> Vec<&mut UploadChunk> {
        self.chunks.iter_mut().collect()
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

#[derive(Deserialize, Serialize)]
pub struct UploadChunk {
    start_range: usize,
    end_range: usize,
    storage_key: Option<String>,
}

impl UploadChunk {
    pub fn new(start_range: usize, end_range: usize) -> Self {
        UploadChunk {
            start_range,
            end_range,
            storage_key: None,
        }
    }
    pub fn start_range(&self) -> usize {
        self.start_range
    }

    pub fn end_range(&self) -> usize {
        self.end_range
    }

    pub fn size(&self) -> usize {
        self.end_range - self.start_range
    }

    pub fn storage_key(&self) -> Option<&str> {
        self.storage_key.as_deref()
    }

    pub fn set_storage_key(&mut self, key: String) -> &mut Self {
        self.storage_key = Some(key);
        self
    }
}

impl Debug for UploadChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UploadChunk")
            .field("start_range", &self.start_range)
            .field("end_range", &self.end_range)
            .field("storage_key", &self.storage_key)
            .finish()
    }
}
