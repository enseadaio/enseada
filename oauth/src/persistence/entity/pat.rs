use chrono::serde::ts_seconds;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;

use crate::session::Session;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PersonalAccessToken {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    label: String,
    session: Session,
    expiration: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl Entity for PersonalAccessToken {
    fn build_guid(id: &str) -> Guid {
        Guid::from(format!("personal_access_token:{}", id))
    }

    fn id(&self) -> &Guid {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}

impl PersonalAccessToken {
    pub fn new(label: String, sig: &str, session: Session, expiration: DateTime<Utc>) -> Self {
        let id = Self::build_guid(sig);
        PersonalAccessToken {
            id,
            rev: None,
            label,
            session,
            expiration,
            revoked_at: None,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn expiration(&self) -> DateTime<Utc> {
        self.expiration
    }

    pub fn expires_in(&self) -> Duration {
        self.expiration.signed_duration_since(Utc::now())
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn revoked_at(&self) -> Option<DateTime<Utc>> {
        self.revoked_at
    }

    pub fn revoke(&mut self) -> &mut Self {
        self.revoked_at = Some(Utc::now());
        self
    }
}
