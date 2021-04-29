#[macro_use]
extern crate lazy_static;

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub use gvk::*;

use crate::core::v1alpha1::{Metadata, TypeMeta};

pub mod core;
pub mod error;
mod gvk;

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

pub mod global {
    use bimap::BiHashMap;
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};
    use tokio::sync::RwLock;

    #[derive(Default)]
    pub struct KindRegistry {
        inner: RwLock<BiHashMap<String, String>>,
    }

    #[derive(Debug, PartialOrd, PartialEq)]
    pub struct KindRegistryError(String);

    impl Display for KindRegistryError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    impl Error for KindRegistryError {}

    impl KindRegistry {
        pub async fn register<K: ToString, KP: ToString>(
            &self,
            kind: K,
            kind_plural: KP,
        ) -> Result<(), KindRegistryError> {
            let mut inner = self.inner.write().await;
            if let Err((kind, kind_plural)) =
                inner.insert_no_overwrite(kind.to_string(), kind_plural.to_string())
            {
                Err(KindRegistryError(format!("failed to register kind '{}' with plural '{}. Either one or the other is already present", kind, kind_plural)))
            } else {
                Ok(())
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[tokio::test]
        async fn it_registers_a_valid_kind() {
            let registry = KindRegistry::default();
            let res = registry.register("Test", "tests").await;
            assert!(res.is_ok());
        }

        #[tokio::test]
        async fn it_fails_to_registers_an_existing_kind() {
            let registry = KindRegistry::default();
            let res = registry.register("Test", "tests").await;
            let res = registry.register("AnotherTest", "tests").await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err(), KindRegistryError("failed to register kind '{}' with plural '{}. Either one or the other is already present".to_string()));
        }
    }
}
