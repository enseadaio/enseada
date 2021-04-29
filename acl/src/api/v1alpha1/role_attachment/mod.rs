use serde::{Deserialize, Serialize};

use api::{KindNamedRef, NamedRef, Resource};
use api::core::v1alpha1::{Metadata, TypeMeta};

use crate::api::v1alpha1::API_VERSION;

pub mod controller;

/*
apiVersion: rbac/v1alpha1
kind: RoleAttachment
metadata:
    name: test-role-test-user
spec:
  roleRef:
    name: test-role
  userRef:
    name: test-user
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleAttachment {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub role_ref: NamedRef,
    pub user_ref: NamedRef,
}

impl Resource for RoleAttachment {
    type Status = ();

    fn type_meta() -> TypeMeta {
        TypeMeta {
            api_version: API_VERSION.clone(),
            kind: "RoleAttachment".to_string(),
            kind_plural: "roleattachments".to_string(),
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

    fn status(&self) -> Option<&Self::Status> {
        None
    }

    fn status_mut(&mut self) -> Option<&mut Self::Status> {
        None
    }

    fn set_status(&mut self, _status: Option<Self::Status>) {}
}
