use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Error, ErrorCode};

mod oci {
    pub mod v1 {
        pub const IMAGE_INDEX: &str = "vnd.oci.image.index.v1+json";
        pub const IMAGE_MANIFEST: &str = "vnd.oci.image.manifest.v1+json";
        pub const IMAGE_LAYER: &str = "vnd.oci.image.layer.v1.tar+gzip";
        pub const IMAGE_CONFIG: &str = "vnd.oci.image.config.v1+json";
    }
}

mod docker {
    pub mod v2 {
        pub const IMAGE_INDEX: &str = "vnd.docker.distribution.manifest.list.v2+json";
        pub const IMAGE_MANIFEST: &str = "vnd.docker.distribution.manifest.v2+json";
        pub const IMAGE_LAYER: &str = "vnd.docker.image.rootfs.diff.tar.gzip";
        pub const IMAGE_CONFIG: &str = "vnd.docker.container.image.v1+json";
    }
}

#[derive(Clone, Debug)]
pub enum MediaType {
    ImageIndex,
    ImageManifest,
    ImageLayer,
    ImageConfig,
}

impl MediaType {
    pub fn name(&self) -> String {
        self.to_string().replace("application/", "")
    }

    pub fn compatible_types(&self) -> Vec<&'static str> {
        match self {
            MediaType::ImageIndex => vec![oci::v1::IMAGE_INDEX, docker::v2::IMAGE_INDEX],
            MediaType::ImageManifest => vec![oci::v1::IMAGE_MANIFEST, docker::v2::IMAGE_MANIFEST],
            MediaType::ImageLayer => vec![oci::v1::IMAGE_LAYER, docker::v2::IMAGE_LAYER],
            MediaType::ImageConfig => vec![oci::v1::IMAGE_CONFIG, docker::v2::IMAGE_CONFIG],
        }
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            MediaType::ImageIndex => oci::v1::IMAGE_INDEX,
            MediaType::ImageManifest => oci::v1::IMAGE_MANIFEST,
            MediaType::ImageLayer => oci::v1::IMAGE_LAYER,
            MediaType::ImageConfig => oci::v1::IMAGE_CONFIG,
        };
        write!(f, "application/{}", s)
    }
}

impl TryFrom<String> for MediaType {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mime = match value.replace("application/", "").as_str() {
            oci::v1::IMAGE_INDEX | docker::v2::IMAGE_INDEX => MediaType::ImageIndex,
            oci::v1::IMAGE_MANIFEST | docker::v2::IMAGE_MANIFEST => MediaType::ImageManifest,
            oci::v1::IMAGE_LAYER | docker::v2::IMAGE_LAYER => MediaType::ImageLayer,
            oci::v1::IMAGE_CONFIG | docker::v2::IMAGE_CONFIG => MediaType::ImageConfig,
            _ => return Err(Error::from(ErrorCode::MediaTypeUnsupported)),
        };
        Ok(mime)
    }
}

impl<'de> Deserialize<'de> for MediaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s).map_err(|err| D::Error::custom(err.to_string()))
    }
}

impl Serialize for MediaType {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        s.serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_parses_a_string() {}
}
