#[macro_use]
extern crate lazy_static;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

pub use api_derive::Resource;
pub use gvk::*;

use crate::core::v1alpha1::{Metadata, TypeMeta};

pub mod core;
pub mod error;
mod gvk;

pub trait Resource: Clone + Debug + DeserializeOwned + Serialize + Send + Sync {
    type Status: Clone + Default + Debug + DeserializeOwned + Serialize;

    fn type_meta() -> TypeMeta;
    fn reset_type_meta(&mut self);
    fn metadata(&self) -> &Metadata;
    fn metadata_mut(&mut self) -> &mut Metadata;
    fn set_metadata(&mut self, metadata: Metadata);
    fn status(&self) -> Option<&Self::Status>;
    fn status_mut(&mut self) -> Option<&mut Self::Status>;
    fn set_status(&mut self, status: Option<Self::Status>);

    fn to_ref(&self) -> NamedRef {
        let name = self.metadata().name.clone();
        NamedRef {
            name,
        }
    }

    fn to_kind_ref(&self) -> KindNamedRef {
        let kind = Self::type_meta().kind;
        let name = self.metadata().name.clone();

        KindNamedRef {
            kind,
            name,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NamedRef {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KindNamedRef {
    pub name: String,
    pub kind: String,
}
