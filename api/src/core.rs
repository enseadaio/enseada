lazy_static! {
    pub static ref API_GROUP: String = "core".to_string();
}

pub mod v1alpha1 {
    lazy_static! {
        pub static ref API_VERSION: GroupVersion = GroupVersion {
            group: super::API_GROUP.clone(),
            version: "v1alpha1".to_string(),
        };
    }

    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};
    use crate::gvk::GroupVersion;
    use std::collections::HashSet;

    #[derive(Clone, Default, Debug, Serialize)]
    pub struct Event<T> {
        pub resource: T,
    }

    impl<T> From<T> for Event<T> {
        fn from(resource: T) -> Self {
            Self { resource }
        }
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct List<T> {
        #[serde(flatten)]
        pub type_meta: TypeMeta,
        pub items: Vec<T>,
        pub limit: usize,
        pub next_token: Option<String>,
    }

    impl<T> List<T> {
        pub fn new(limit: usize, next_token: Option<String>, items: Vec<T>) -> Self {
            Self {
                type_meta: TypeMeta {
                    api_version: API_VERSION.clone(),
                    kind: "List".to_string(),
                    kind_plural: "lists".to_string(),
                },
                items,
                next_token,
                limit,
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TypeMeta {
        pub api_version: GroupVersion,
        pub kind: String,
        #[serde(skip)]
        pub kind_plural: String,
    }

    #[derive(Clone, Default, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Metadata {
        pub name: String,
        #[serde(default)]
        pub created_at: Option<DateTime<Utc>>,
        #[serde(default)]
        pub deleted_at: Option<DateTime<Utc>>,
        #[serde(default)]
        pub finalizers: HashSet<String>,
    }

    impl Metadata {
        pub fn named<N: ToString>(name: N) -> Self {
            Self {
                name: name.to_string(),
                ..Default::default()
            }
        }
        pub fn is_just_created(&self) -> bool {
            self.created_at.is_none()
        }

        pub fn is_deleted(&self) -> bool {
            self.deleted_at.is_some()
        }

        pub fn has_finalizer(&self, finalizer: &str) -> bool {
            self.finalizers.contains(finalizer)
        }

        pub fn set_finalizer<F: ToString>(&mut self, finalizer: F) {
            self.finalizers.insert(finalizer.to_string());
        }

        pub fn remove_finalizer(&mut self, finalizer: &str) {
            self.finalizers.remove(finalizer);
        }
    }
}
