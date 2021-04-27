#[macro_use]
extern crate lazy_static;

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::core::v1alpha1::{Metadata, TypeMeta};

pub mod core;
pub mod error;

pub trait Resource: Clone + Default + Debug + DeserializeOwned + Serialize + Send + Sync {
    type Status: Clone + Default + Debug + DeserializeOwned + Serialize;

    fn type_meta() -> TypeMeta;
    fn metadata(&self) -> &Metadata;
    fn metadata_mut(&mut self) -> &mut Metadata;
    fn set_metadata(&mut self, metadata: Metadata);
    fn status(&self) -> Option<&Self::Status>;
    fn status_mut(&mut self) -> Option<&mut Self::Status>;
    fn set_status(&mut self, status: Option<Self::Status>);
}
