use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::containers::error::Error;
use crate::containers::name::Name;
use crate::containers::Result;
use crate::couchdb;
use crate::couchdb::db;
use crate::couchdb::db::Database;
use crate::guid::Guid;
use crate::pagination::{Cursor, Page};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Repo {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    group: String,
    name: String,
}

impl Repo {
    pub fn build_guid(uuid: &str) -> Guid {
        Guid::partitioned("repo", &uuid)
    }

    pub fn new(name: Name) -> Self {
        let id = Self::build_guid(&Uuid::new_v4().to_string());
        Repo {
            id,
            rev: None,
            group: name.group().to_string(),
            name: name.name().to_string(),
        }
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn rev(&self) -> Option<String> {
        self.rev.clone()
    }

    pub fn name(&self) -> Name {
        Name::new(self.group.clone(), self.name.clone())
    }

    pub fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}

pub struct RepoService {
    db: Database,
}

impl RepoService {
    pub fn new(db: Database) -> RepoService {
        RepoService { db }
    }

    pub async fn list_repos(&self, limit: usize, cursor: Option<&Cursor>) -> Result<Page<Repo>> {
        let res = self.db.list::<Repo>("repo", limit + 1, cursor.map(Cursor::to_string)).await?;
        Ok(Page::from_rows_response(res, limit))
    }

    pub async fn find_repo(&self, id: &str) -> Result<Option<Repo>> {
        let guid = Repo::build_guid(id).to_string();
        self.db.get(guid.as_str()).await.map_err(Error::from)
    }

    pub async fn find_repo_by_name(&self, name: &Name) -> Result<Option<Repo>> {
        log::debug!("Finding oci repo with name {}", name);
        let response = self.db.find_partitioned("repo", serde_json::json!({
            "group": name.group(),
            "name": name.name(),
        }), 1, None).await?;
        if let Some(warning) = &response.warning {
            log::warn!("{}", warning);
        }

        log::debug!("Found {} repos with name {}", response.docs.len(), name);
        let repo = response.docs.first().cloned();
        Ok(repo)
    }

    pub async fn save_repo(&self, repo: Repo) -> Result<Repo> {
        let guid = repo.id().to_string();
        let res = self.db.put(guid.as_str(), &repo).await?;
        let mut repo = repo;
        repo.set_rev(res.rev);
        Ok(repo)
    }

    pub async fn delete_repo(&self, repo: &Repo) -> Result<()> {
        let id = repo.id().to_string();
        let rev = match repo.rev() {
            Some(rev) => rev,
            None => panic!("repo {} is missing rev", id),
        };
        self.db.delete(&id, &rev).await.map_err(Error::from)
    }
}

pub fn add_repo_service(app: &mut ServiceConfig) {
    let couch = &couchdb::SINGLETON;
    let db = couch.database(db::name::OCI, true);
    let service = RepoService::new(db);
    app.data(service);
}
