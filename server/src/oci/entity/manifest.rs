use serde::{Deserialize, Serialize};

use enseada::guid::Guid;

use crate::couchdb::repository::Entity;
use crate::oci::manifest::ImageManifest;
use crate::oci::mime::MediaType;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    manifest: ImageManifest,
}

impl Manifest {
    pub fn new(reference: &str, manifest: ImageManifest) -> Self {
        let mut manifest = Self::from(manifest);
        manifest.id = Self::build_guid(reference);
        manifest
    }

    pub fn into_inner(self) -> ImageManifest {
        self.manifest
    }
}

impl From<ImageManifest> for Manifest {
    fn from(manifest: ImageManifest) -> Self {
        Self {
            id: Self::build_guid(&manifest.digest().to_string()),
            rev: None,
            manifest,
        }
    }
}

impl Entity for Manifest {
    fn build_guid(id: &str) -> Guid {
        Guid::partitioned(&MediaType::ImageManifest.name(), id)
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
