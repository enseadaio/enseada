use serde::{Deserialize, Serialize};

use api::{KindNamedRef, NamedRef, Resource};
use api::core::v1alpha1::{Metadata, TypeMeta};
pub use controller::PolicyAttachmentController;

mod controller;

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

