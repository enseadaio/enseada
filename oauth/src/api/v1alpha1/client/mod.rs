use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use api::core::v1alpha1::{Metadata, TypeMeta};
use api::Resource;
pub use controller::*;

use crate::scope::Scope;

use super::API_VERSION;

mod controller;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ClientType {
    Public,
    // Confidential, // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthClient {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub spec: OAuthClientSpec,
    pub status: Option<OAuthClientStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthClientSpec {
    client_type: ClientType,
    allowed_scopes: Scope,
    allowed_redirect_uris: HashSet<url::Url>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthClientStatus {
    condition: OAuthClientCondition,
    #[serde(default)]
    condition_message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum OAuthClientCondition {
    Pending,
    Active,
    Error,
}

impl Default for OAuthClientCondition {
    fn default() -> Self {
        Self::Pending
    }
}

impl Resource for OAuthClient {
    type Status = OAuthClientStatus;

    fn type_meta() -> TypeMeta {
        TypeMeta {
            api_version: API_VERSION.clone(),
            kind: "OAuthClient".to_string(),
            kind_plural: "oauthclients".to_string(),
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
        self.status.as_ref()
    }

    fn status_mut(&mut self) -> Option<&mut Self::Status> {
        if self.status.is_none() {
            self.status = Some(Default::default());
        }

        self.status.as_mut()
    }

    fn set_status(&mut self, status: Option<Self::Status>) {
        self.status = status;
    }
}
