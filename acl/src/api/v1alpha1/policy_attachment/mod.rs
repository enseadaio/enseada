use serde::{Deserialize, Serialize};

use api::{KindNamedRef, NamedRef, Resource};
use api::core::v1alpha1::{Metadata, TypeMeta};
pub use controller::PolicyAttachmentController;

use crate::api::v1alpha1::API_VERSION;

mod controller;
/*
apiVersion: rbac/v1alpha1
kind: PolicyAttachment
metadata:
    name: test-role
spec:
  policyRef:
    name: test
  subjects:
    - name: test-role
      kind: Role
    - name: test-user
      kind: User
 */
#[derive(Clone, Debug, Deserialize, Serialize, Resource)]
#[serde(rename_all = "camelCase")]
#[resource(api_version = "acl/v1alpha1", kind = "PolicyAttachment", kind_plural = "policyattachments")]
pub struct PolicyAttachment {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub policy_ref: NamedRef,
    pub subjects: Vec<KindNamedRef>,
}

