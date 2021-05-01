use serde::{Deserialize, Serialize};

use api::{NamedRef, Resource};
use api::core::v1alpha1::{Metadata, TypeMeta};
pub use controller::RoleAttachmentController;

use crate::api::v1alpha1::API_VERSION;

mod controller;
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
#[derive(Clone, Debug, Deserialize, Serialize, Resource)]
#[serde(rename_all = "camelCase")]
#[resource(api_version = "acl/v1alpha1", kind = "RoleAttachment", kind_plural = "roleattachments")]
pub struct RoleAttachment {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub role_ref: NamedRef,
    pub user_ref: NamedRef,
}

