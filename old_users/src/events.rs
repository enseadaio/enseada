use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;
use events::Event;

use crate::User;

#[derive(Debug, Event)]
pub struct UserCreated {
    pub id: Guid,
    pub rev: Option<String>,
    pub enabled: bool,
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
    pub id: Guid,
    pub rev: Option<String>,
    pub enabled: bool,
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
    pub id: Guid,
    pub rev: Option<String>,
    pub enabled: bool,
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
