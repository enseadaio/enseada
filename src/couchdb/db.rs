use std::fmt::Debug;
use std::sync::Arc;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::couchdb::client::Client;
use crate::couchdb::error::Error;
use crate::couchdb::responses;
use crate::couchdb::responses::{FindResponse, PutResponse, RowsResponse};
use crate::couchdb::Result;

pub mod name {
    pub const OAUTH: &str = "oauth";
    pub const USERS: &str = "users";
    pub const RBAC: &str = "rbac";
}

pub struct Database {
    client: Arc<Client>,
    name: String,
    partitioned: bool,
}

impl Database {
    pub(super) fn new(client: Arc<Client>, name: String, partitioned: bool) -> Database {
        Database { client, name, partitioned }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub async fn get_self(&self) -> Result<responses::DBInfo> {
        log::debug!("Getting info for database {}", self.name);
        self.client.get(self.name.as_str(), None::<bool>).await
            .map_err(Error::from)
    }

    pub async fn create_self(&self) -> Result<bool> {
        log::debug!("Creating database {}", &self.name);
        let res: responses::Ok = self.client
            .put(self.name.as_str(), None::<bool>, Some(&[("partitioned", &self.partitioned)]))
            .await?;
        Ok(res.ok)
    }

    pub async fn get<R: DeserializeOwned>(&self, id: &str) -> Result<Option<R>> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Getting {} from couch", &path);
        match self.client.get(path.as_str(), None::<bool>).await {
            Ok(r) => Ok(Some(r)),
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Ok(None),
                _ => Err(Error::from(err)),
            }
        }
    }

    pub async fn list<R: DeserializeOwned + Clone>(&self, kind: &str, limit: usize, offset: usize) -> Result<RowsResponse<R>> {
        let path = format!("{}/_partition/{}/_all_docs", &self.name, kind);
        let query = Some(ListQuery {
            include_docs: true,
            limit,
            skip: offset,
        });
        self.client.get(path.as_str(), query).await
            .map_err(Error::from)
    }

    pub async fn list_all<R: DeserializeOwned + Clone>(&self, kind: &str) -> Result<RowsResponse<R>> {
        let path = format!("{}/_partition/{}/_all_docs", &self.name, kind);
        self.client.get(path.as_str(), Some(&[("include_docs", true)])).await
            .map_err(Error::from)
    }

    pub async fn put<T: Serialize + Debug>(&self, id: &str, entity: T) -> Result<PutResponse> {
        let path = format!("{}/{}", &self.name, &id);
        log::debug!("Putting {} into couch: {:?}", &path, &entity);
        self.client.put(path.as_str(), Some(entity), None::<usize>).await
            .map_err(|err| match err.status() {
                Some(StatusCode::CONFLICT) => Error::map_message(err, &format!("document {} already exists in database {}", &id, &self.name)),
                _ => Error::from(err)
            })
    }

    pub async fn find<R: DeserializeOwned>(&self, selector: serde_json::Value) -> Result<FindResponse<R>> {
        let path = format!("{}/_find", &self.name);
        let body = serde_json::json!({
            "selector": selector,
        });

        log::debug!("Finding from {} with query {}", &self.name, &body);

        self.client.post(path.as_str(), Some(body), None::<bool>).await
            .map_err(Error::from)
    }

    pub async fn delete(&self, id: &str, rev: &str) -> Result<()> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Deleting {} from couch", &path);
        self.client.delete(path.as_str(), Some(&[("rev", rev)])).await?;
        Ok(())
    }

    pub async fn exists(&self, id: &str) -> Result<bool> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Checking {} existence from couch", &path);
        let res = self.client.exists(path.as_str()).await?;
        Ok(res)
    }
}

#[derive(Serialize)]
struct ListQuery {
    pub include_docs: bool,
    pub limit: usize,
    pub skip: usize,
}
