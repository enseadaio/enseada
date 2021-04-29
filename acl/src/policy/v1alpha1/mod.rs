use serde::{Deserialize, Serialize};

use api::core::v1alpha1::{Metadata, TypeMeta};
use api::{GroupVersion, Resource};

pub mod controller;

lazy_static! {
    pub static ref API_VERSION: GroupVersion = GroupVersion {
        group: "rbac".to_string(),
        version: "v1alpha1".to_string(),
    };
}

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
    pub api_groups: Vec<String>,
    pub resources: Vec<String>,
    pub resource_names: Option<Vec<String>>,
    pub actions: Vec<String>,
}
