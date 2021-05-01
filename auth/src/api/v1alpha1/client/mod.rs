use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use api::core::v1alpha1::{Metadata, TypeMeta};
use api::Resource;
pub use controller::*;
use oauth::scope::Scope;

mod controller;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ClientType {
    Public,
    // Confidential, // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize, Resource)]
#[serde(rename_all = "camelCase")]
#[resource(api_version = "auth/v1alpha1", kind = "OAuthClient", kind_plural = "oauthclients", status = "OAuthClientStatus")]
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
