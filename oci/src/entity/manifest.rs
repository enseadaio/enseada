use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;

use crate::manifest::ImageManifest;
use crate::mime::MediaType;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    manifest: ImageManifest,
    image: String,
}

impl Manifest {
    pub fn new(reference: &str, group: &str, name: &str, manifest: ImageManifest) -> Self {
        let mut manifest = Self::from(manifest);
        manifest.id = Self::build_guid(reference);
        manifest.image = format!("{}/{}", group, name);
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
            image: String::new(),
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
