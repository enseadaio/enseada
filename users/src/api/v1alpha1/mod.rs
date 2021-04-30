use serde::{Deserialize, Serialize};

use api::{core, Resource};
use api::core::v1alpha1::{Metadata, TypeMeta};
pub use controller::UserController;

mod controller;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub spec: UserSpec,
    pub status: Option<UserStatus>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSpec {
    pub enabled: bool,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatus {
    pub enabled: bool,
}

impl Resource for User {
    type Status = UserStatus;

    fn type_meta() -> TypeMeta {
        TypeMeta {
            api_version: core::v1alpha1::API_VERSION.clone(),
            kind: "User".to_string(),
            kind_plural: "users".to_string(),
        }
    }

    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }

    fn status(&self) -> Option<&UserStatus> {
        self.status.as_ref()
    }

    fn status_mut(&mut self) -> Option<&mut UserStatus> {
        if self.status.is_none() {
            self.status = Some(Default::default());
        }

        self.status.as_mut()
    }

    fn set_status(&mut self, status: Option<Self::Status>) {
        self.status = status;
    }
}
