use enseada::couchdb::repository::Entity;
use enseada::events::Event;
use enseada::guid::Guid;

use crate::User;

#[derive(Debug)]
pub struct UserCreated {
    id: Guid,
    rev: Option<String>,
    enabled: bool,
}

impl Event for UserCreated {}

impl From<&User> for UserCreated {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().clone(),
            rev: user.rev().map(str::to_string),
            enabled: user.is_enabled(),
        }
    }
}

#[derive(Debug)]
pub struct UserUpdated {
    id: Guid,
    rev: Option<String>,
    enabled: bool,
}

impl Event for UserUpdated {}

impl From<&User> for UserUpdated {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().clone(),
            rev: user.rev().map(str::to_string),
            enabled: user.is_enabled(),
        }
    }
}

#[derive(Debug)]
pub struct UserDeleted {
    id: Guid,
    rev: Option<String>,
    enabled: bool,
}

impl Event for UserDeleted {}

impl From<&User> for UserDeleted {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().clone(),
            rev: user.rev().map(str::to_string),
            enabled: user.is_enabled(),
        }
    }
}
