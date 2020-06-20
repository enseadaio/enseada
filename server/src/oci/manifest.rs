use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::oci::digest::Digest;
use crate::oci::mime::MediaType;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    media_type: MediaType,
    digest: Digest,
    #[serde(default)]
    size: usize,
    urls: Option<Vec<Url>>,
    annotations: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageManifest {
    schema_version: u8,
    config: Descriptor,
    layers: Vec<Descriptor>,
    annotations: Option<HashMap<String, String>>,
}

impl ImageManifest {
    pub fn new(digest: Digest, size: usize) -> Self {
        Self {
            schema_version: 2,
            config: Descriptor {
                media_type: MediaType::ImageConfig,
                digest,
                size,
                urls: None,
                annotations: None,
            },
            layers: Vec::new(),
            annotations: None,
        }
    }

    pub fn digest(&self) -> &Digest {
        &self.config.digest
    }

    pub fn add_layer(&mut self, digest: Digest, size: usize) -> &mut Self {
        let layer = Descriptor {
            media_type: MediaType::ImageLayer,
            digest,
            size,
            urls: None,
            annotations: None,
        };
        self.layers.push(layer);
        self
    }
}
