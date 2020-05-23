use serde::{Deserialize, Serialize};
use url::Url;

use crate::containers::digest::Digest;
use crate::containers::mime::oci::v1::{IMAGE_CONFIG, IMAGE_INDEX, IMAGE_LAYER, IMAGE_MANIFEST};
use crate::containers::name::Name;
use crate::guid::Guid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageIndex {
    schema_version: u8,
    media_type: String,
    manifests: Vec<ManifestObject>,
}

impl ImageIndex {
    pub fn build_guid(name: &Name) -> Guid {
        Guid::partitioned(IMAGE_INDEX, &name.to_string())
    }

    pub fn new(manifests: Vec<ManifestObject>) -> Self {
        ImageIndex {
            schema_version: 2,
            media_type: IMAGE_INDEX.to_string(),
            manifests,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestObject {
    media_type: String,
    size: u64,
    digest: Digest,
    platform: Platform,
}

impl ManifestObject {
    pub fn new(size: u64, digest: Digest, platform: Platform) -> Self {
        ManifestObject {
            media_type: IMAGE_MANIFEST.to_string(),
            size,
            digest,
            platform,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub architecture: String,
    pub os: String,
    #[serde(rename = "os.version", skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(rename = "os.features", skip_serializing_if = "Vec::is_empty")]
    pub os_features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageManifest {
    schema_version: u8,
    media_type: String,
    config: ImageManifestConfig,
    layers: Vec<ImageManifestLayer>
}

impl ImageManifest {
    pub fn build_guid(name: &Name) -> Guid {
        Guid::partitioned(IMAGE_MANIFEST, &name.to_string())
    }
    
    pub fn new(config: ImageManifestConfig, layers: Vec<ImageManifestLayer>) -> Self {
        ImageManifest {
            schema_version: 2,
            media_type: IMAGE_MANIFEST.to_string(),
            config,
            layers,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageManifestConfig {
    media_type: String,
    size: u64,
    digest: Digest,
}

impl ImageManifestConfig {
    pub fn new(size: u64, digest: Digest) -> Self {
        ImageManifestConfig {
            media_type: IMAGE_CONFIG.to_string(),
            size,
            digest,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageManifestLayer {
    media_type: String,
    size: u64,
    digest: Digest,
    urls: Vec<Url>,
}

impl ImageManifestLayer {
    pub fn new(size: u64, digest: Digest, urls: Vec<Url>) -> Self {
        ImageManifestLayer {
            media_type: IMAGE_LAYER.to_string(),
            size,
            digest,
            urls,
        }
    }
}