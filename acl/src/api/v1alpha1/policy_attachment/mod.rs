use serde::{Deserialize, Serialize};

use api::{KindNamedRef, NamedRef, Resource};
use api::core::v1alpha1::{Metadata, TypeMeta};

use crate::api::v1alpha1::API_VERSION;

pub mod controller;

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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyAttachment {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub policy_ref: NamedRef,
    pub subjects: Vec<KindNamedRef>,
}

impl Resource for PolicyAttachment {
    type Status = ();

    fn type_meta() -> TypeMeta {
        TypeMeta {
            api_version: API_VERSION.clone(),
            kind: "PolicyAttachment".to_string(),
            kind_plural: "policyattachments".to_string(),
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
