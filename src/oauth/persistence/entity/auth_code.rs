use serde::{Deserialize, Serialize};
use crate::oauth::client::Client;
use crate::couchdb::guid::Guid;
use crate::oauth::Scope;
use url::Url;
use uuid::Uuid;
use crate::oauth::code::AuthorizationCode;
use crate::oauth::session::Session;
use crate::secure::SecureSecret;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthorizationCodeEntity {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    sig: String,
    session: Session,
}

impl AuthorizationCodeEntity {
    pub fn build_guid(id: String) -> Guid {
        Guid::from(format!("code:{}", id))
    }
    pub fn new(sig: String, session: Session) -> AuthorizationCodeEntity {
        let id = Self::build_guid(Uuid::new_v4().to_string());
        AuthorizationCodeEntity { id, rev: None::<String>, sig, session, }
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn rev(&self) -> Option<String> {
        self.rev.clone()
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn to_empty_code(&self) -> AuthorizationCode {
        AuthorizationCode::new(SecureSecret::empty(), self.session().clone())
    }
}
