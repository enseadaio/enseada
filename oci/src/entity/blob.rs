use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;

use crate::digest::Digest;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Blob {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    digest: Digest,
}

impl Blob {
    pub fn new(digest: Digest) -> Self {
        Self {
            id: Self::build_guid(&digest.to_string()),
            rev: None,
            digest,
        }
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

impl Entity for Blob {
    fn build_guid(id: &str) -> Guid {
        Guid::partitioned("oci_blob", id)
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
