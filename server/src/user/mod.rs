use std::fmt;
use std::fmt::Debug;

use serde::export::Formatter;
use serde::{Deserialize, Serialize};

use couchdb::db::Database;
use enseada::error::Error;
use enseada::guid::Guid;
use enseada::pagination::{Cursor, Page};
use enseada::secure;
pub use service::{ListUsers, UserService};

mod service;

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    password_hash: String,
}

impl User {
    pub fn build_guid(username: &str) -> Guid {
        Guid::partitioned("user", username)
    }

    pub fn new(username: String, password: String) -> Result<User, Error> {
        let password_hash = secure::hash_password(password.as_str())?;
        let id = Self::build_guid(&username);
        Ok(User {
            id,
            rev: None,
            password_hash,
        })
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn rev(&self) -> Option<String> {
        self.rev.clone()
    }

    pub fn username(&self) -> &str {
        self.id.id()
    }

    pub fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}

impl Debug for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "User {{ id: {:?}, rev: {:?} }}", &self.id, &self.rev)
    }
}
