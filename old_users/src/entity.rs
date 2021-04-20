use std::fmt;
use std::fmt::Debug;

use serde::export::Formatter;
use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::error::Error;
use enseada::guid::Guid;
use enseada::secure;

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    password_hash: String,
    enabled: bool,
}

impl User {
    pub fn new(username: String, password: String) -> Result<User, Error> {
        let password_hash = secure::hash_password(password.as_str())?;
        let id = Self::build_guid(&username);
        Ok(User {
            id,
            rev: None,
            password_hash,
            enabled: true,
        })
    }

    pub fn username(&self) -> &str {
        self.id.id()
    }

    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = enabled;
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(super) fn password_hash(&self) -> &str {
        &self.password_hash
    }
}

impl Entity for User {
    fn build_guid(username: &str) -> Guid {
        Guid::partitioned("user", username)
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

impl Debug for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("rev", &self.rev)
            .field("enabled", &self.enabled)
            .finish()
    }
}
