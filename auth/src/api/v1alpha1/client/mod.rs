use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use api::core::v1alpha1::{Metadata, TypeMeta};
use api::Resource;
pub use controller::*;
use oauth::scope::Scope;
use oauth::client::{Client, ClientKind};

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
    pub client_type: ClientType,
    pub allowed_scopes: Scope,
    pub allowed_redirect_uris: HashSet<url::Url>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthClientStatus {
    pub condition: OAuthClientCondition,
    #[serde(default)]
    pub condition_message: String,
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

impl Into<Client> for OAuthClient {
    fn into(self) -> Client {
        let kind = match self.spec.client_type {
            ClientType::Public => ClientKind::Public,
        };

        Client::new(self.metadata.name, kind, self.spec.allowed_scopes, self.spec.allowed_redirect_uris)
    }
}
