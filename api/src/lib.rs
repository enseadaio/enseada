use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::core::v1alpha1::{Metadata, TypeMeta};

pub mod core;
pub mod error;
pub mod users;

pub trait Resource: Clone + Default + Debug + DeserializeOwned + Serialize {
    fn type_meta(&self) -> &TypeMeta;
    fn metadata(&self) -> &Metadata;
    fn metadata_mut(&mut self) -> &mut Metadata;
}

