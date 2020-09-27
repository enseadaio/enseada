use enseada::couchdb::repository::Entity;
use events::Event;
use enseada::guid::Guid;

use crate::User;

#[derive(Debug, Event)]
pub struct UserCreated {
    id: Guid,
    rev: Option<String>,
    enabled: bool,
}

impl From<&User> for UserCreated {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().clone(),
            rev: user.rev().map(str::to_string),
            enabled: user.is_enabled(),
        }
    }
}

#[derive(Debug, Event)]
pub struct UserUpdated {
    id: Guid,
    rev: Option<String>,
    enabled: bool,
}

impl From<&User> for UserUpdated {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().clone(),
            rev: user.rev().map(str::to_string),
            enabled: user.is_enabled(),
        }
    }
}

#[derive(Debug, Event)]
pub struct UserDeleted {
    id: Guid,
    rev: Option<String>,
    enabled: bool,
}

impl From<&User> for UserDeleted {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().clone(),
            rev: user.rev().map(str::to_string),
            enabled: user.is_enabled(),
        }
    }
}
