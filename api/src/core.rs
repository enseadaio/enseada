pub mod v1alpha1 {
    lazy_static! {
        pub static ref API_VERSION: GroupVersion = GroupVersion {
            group: "core".to_string(),
            version: "v1alpha1".to_string(),
        };
    }

    use serde::{Deserialize, Serialize, Deserializer, Serializer};
    use chrono::{DateTime, Utc};
    use serde::de::Error;
    use std::fmt::{self, Display, Formatter};

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
    }

    impl<T> Default for List<T> {
        fn default() -> Self {
            Self {
                type_meta: TypeMeta {
                    api_version: API_VERSION.clone(),
                    kind: "List".to_string(),
                    kind_plural: "lists".to_string(),
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

    #[derive(Clone, Debug)]
    pub struct GroupVersion {
        pub group: String,
        pub version: String,
    }

    impl<'de> Deserialize<'de > for GroupVersion {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
            D: Deserializer<'de> {
            let s = String::deserialize(deserializer)?;
            let parts: Vec<&str> = s.split('/').collect();

            if let [group, version, ..] = parts.as_slice() {
                Ok(Self {
                    group: group.to_string(),
                    version: version.to_string(),
                })
            } else {
                Err(<D as Deserializer<'de>>::Error::custom(format!("invalid GroupVersion '{}'", s)))
            }
        }
    }

    impl Serialize for GroupVersion {
        fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
            S: Serializer {
            self.to_string().serialize(serializer)
        }
    }

    impl Display for GroupVersion {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}/{}", self.group, self.version)
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
