use serde::{Deserialize, Serialize};
use url::Url;

use crate::docker::mime::{FOREIGN_LAYER, IMAGE_MANIFEST_V2, IMAGE_V1, LAYER, MANIFEST_LIST_V2};
use crate::guid::Guid;
use crate::docker::Name;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestList {
    schema_version: u8,
    media_type: String,
    manifests: Vec<ManifestObject>,
}

impl ManifestList {
    pub fn build_guid(name: &Name) -> Guid {
        Guid::partitioned("docker_list_v2", &format!("{}:{}", name.group, name.name))
    }

    pub fn new(manifests: Vec<ManifestObject>) -> Self {
        ManifestList {
            schema_version: 2,
            media_type: MANIFEST_LIST_V2.to_string(),
            manifests,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestObject {
    media_type: String,
    size: u64,
    digest: String,
    platform: Platform,
}

impl ManifestObject {
    pub fn new(size: u64, digest: String, platform: Platform) -> Self {
        ManifestObject {
            media_type: IMAGE_MANIFEST_V2.to_string(),
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
        Guid::partitioned("docker_image_v2", &format!("{}:{}", name.group, name.name))
    }
    
    pub fn new(config: ImageManifestConfig, layers: Vec<ImageManifestLayer>) -> Self {
        ImageManifest {
            schema_version: 2,
            media_type: IMAGE_MANIFEST_V2.to_string(),
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
    digest: String,
}

impl ImageManifestConfig {
    pub fn new(size: u64, digest: String) -> Self {
        ImageManifestConfig {
            media_type: IMAGE_V1.to_string(),
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
    digest: String,
    urls: Vec<Url>,
}

impl ImageManifestLayer {
    pub fn new(remote: bool, size: u64, digest: String, urls: Vec<Url>) -> Self {
        ImageManifestLayer {
            media_type: if remote { FOREIGN_LAYER } else { LAYER }.to_string(),
            size,
            digest,
            urls,
        }
    }
}