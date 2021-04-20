pub mod v1alpha1 {
    use serde::{Deserialize, Serialize};

    use crate::core;
    use crate::core::v1alpha1::{Metadata, TypeMeta};
    use crate::Resource;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct User {
        #[serde(flatten)]
        pub type_meta: TypeMeta,
        pub metadata: Metadata,
        pub spec: UserSpec,
        pub status: Option<UserStatus>,
    }

    impl Default for User {
        fn default() -> Self {
            Self {
                type_meta: TypeMeta {
                    api_version: core::v1alpha1::API_VERSION.to_string(),
                    kind: "User".to_string(),
                },
                metadata: Default::default(),
                spec: Default::default(),
                status: Default::default(),
            }
        }
    }

    impl User {
        pub fn status_mut(&mut self) -> &mut UserStatus {
            if self.status.is_none() {
                self.status = Some(Default::default());
            }

            self.status.as_mut().unwrap()
        }
    }

    #[derive(Clone, Default, Debug, Deserialize, Serialize)]
    pub struct UserSpec {
        pub enabled: bool,
    }

    #[derive(Clone, Default, Debug, Deserialize, Serialize)]
    pub struct UserStatus {
        pub enabled: bool
    }

    impl Resource for User {
        fn type_meta(&self) -> &TypeMeta {
            &self.type_meta
        }

        fn metadata(&self) -> &Metadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.metadata
        }
    }
}
