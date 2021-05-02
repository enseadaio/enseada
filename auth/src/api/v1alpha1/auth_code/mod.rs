use serde::{Deserialize, Serialize};

use api::core::v1alpha1::{Metadata, TypeMeta};
use api::{Resource, SecureResource};
pub use controller::*;
use controller_runtime::{DateTime, Utc};
use oauth::request::PkceRequest;
use oauth::session::Session;
use crypto::SecureSecret;

mod controller;

#[derive(Clone, Debug, Deserialize, Serialize, Resource)]
#[serde(rename_all = "camelCase")]
#[resource(api_version = "auth/v1alpha1", kind = "OAuthAuthorizationCode", kind_plural = "oauthauthorizationcodes")]
pub struct OAuthAuthorizationCode {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub session: Session,
    pub expiration: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkce: Option<PkceRequest>,
    pub code_hash: SecureSecret,
}
