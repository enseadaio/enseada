use enseada::couchdb::repository::Entity;
use enseada::events::Event;
use enseada::guid::Guid;

use crate::entity::Repo;

#[derive(Debug)]
pub struct RepoCreated {
    pub id: Guid,
    pub rev: Option<String>,
    pub group: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

impl Event for RepoCreated {}

impl From<&Repo> for RepoCreated {
    fn from(repo: &Repo) -> Self {
        Self {
            id: repo.id().clone(),
            rev: repo.rev().map(str::to_string),
            group: repo.group().to_string(),
            name: repo.name().to_string(),
            description: repo.description().map(str::to_string),
            tags: repo.tags().clone(),
        }
    }
}

#[derive(Debug)]
pub struct RepoUpdated {
    pub id: Guid,
    pub rev: Option<String>,
    pub group: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

impl Event for RepoUpdated {}

impl From<&Repo> for RepoUpdated {
    fn from(repo: &Repo) -> Self {
        Self {
            id: repo.id().clone(),
            rev: repo.rev().map(str::to_string),
            group: repo.group().to_string(),
            name: repo.name().to_string(),
            description: repo.description().map(str::to_string),
            tags: repo.tags().clone(),
        }
    }
}

#[derive(Debug)]
pub struct RepoDeleted {
    pub id: Guid,
    pub rev: Option<String>,
    pub group: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

impl Event for RepoDeleted {}

impl From<&Repo> for RepoDeleted {
    fn from(repo: &Repo) -> Self {
        Self {
            id: repo.id().clone(),
            rev: repo.rev().map(str::to_string),
            group: repo.group().to_string(),
            name: repo.name().to_string(),
            description: repo.description().map(str::to_string),
            tags: repo.tags().clone(),
        }
    }
}
