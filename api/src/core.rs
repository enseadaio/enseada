pub mod v1alpha1 {
    pub const API_VERSION: &str = "core/v1alpha1";

    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};

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
    pub struct List<T> {
        #[serde(flatten)]
        pub type_meta: TypeMeta,
        pub items: Vec<T>,
    }

    impl<T> Default for List<T> {
        fn default() -> Self {
            Self {
                type_meta: TypeMeta {
                    api_version: API_VERSION.to_string(),
                    kind: "List".to_string(),
                },
                items: Vec::default(),
            }
        }
    }

    impl<T> From<Vec<T>> for List<T> {
        fn from(items: Vec<T>) -> Self {
            Self {
                items,
                ..Default::default()
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TypeMeta {
        pub api_version: String,
        pub kind: String,
    }

    #[derive(Clone, Default, Debug, Deserialize, Serialize)]
    pub struct Metadata {
        pub name: String,
        pub created_at: Option<DateTime<Utc>>,
        pub deleted_at: Option<DateTime<Utc>>,
    }

    impl Metadata {
        pub fn is_just_created(&self) -> bool {
            self.created_at.is_none()
        }

        pub fn is_deleted(&self) -> bool {
            self.deleted_at.is_some()
        }
    }
}
