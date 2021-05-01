use serde::{Deserialize, Serialize};

use api::{GroupKindName, Resource};
use api::core::v1alpha1::{Metadata, TypeMeta};
pub use controller::PolicyController;

mod controller;
/*
apiVersion: rbac/v1alpha1
kind: Policy
metadata:
    name: test
rules:
- resources: ['* / * / *']
  actions: ['*']
 */
#[derive(Clone, Debug, Deserialize, Serialize, Resource)]
#[serde(rename_all = "camelCase")]
#[resource(api_version = "acl/v1alpha1", kind = "Policy", kind_plural = "policies")]
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


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub resources: Vec<GroupKindName>,
    pub actions: Vec<String>,
}
