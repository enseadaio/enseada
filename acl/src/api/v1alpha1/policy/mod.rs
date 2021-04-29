use serde::{Deserialize, Serialize};

use api::{GroupVersionKind, Resource, GroupVersionKindName};
use api::core::v1alpha1::{Metadata, TypeMeta};

use crate::api::v1alpha1::API_VERSION;

pub mod controller;

/*
apiVersion: rbac/v1alpha1
kind: Policy
metadata:
    name: test
rules:
- resources: ['* / * / *']
  actions: ['*']
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub rules: Vec<Rule>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            type_meta: <Policy as Resource>::type_meta(),
            metadata: Default::default(),
            rules: Vec::new(),
        }
    }
}

impl Resource for Policy {
    type Status = ();

    fn type_meta() -> TypeMeta {
        TypeMeta {
            api_version: API_VERSION.clone(),
            kind: "Policy".to_string(),
            kind_plural: "policies".to_string(),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub resources: Vec<GroupVersionKindName>,
    pub actions: Vec<String>,
}
