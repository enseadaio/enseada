use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;

use crate::manifest::ImageManifest;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    manifest: ImageManifest,
    image: String,
    reference: String,
}

impl Manifest {
    pub fn new(group: &str, name: &str, reference: &str, manifest: ImageManifest) -> Self {
        Self {
            id: Self::build_guid(&Self::build_id(group, name, reference)),
            rev: None,
            image: format!("{}/{}", group, name),
            reference: reference.to_string(),
            manifest,
        }
    }

    pub fn into_inner(self) -> ImageManifest {
        self.manifest
    }

    pub fn build_id(group: &str, name: &str, reference: &str) -> String {
        format!("{}-{}:{}", group, name, reference)
    }
}

impl Entity for Manifest {
    fn build_guid(id: &str) -> Guid {
        Guid::partitioned("oci_manifest", id)
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
