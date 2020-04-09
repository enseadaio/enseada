use serde::de::DeserializeOwned;

use crate::couchdb::client::Client;
use crate::couchdb::responses;
use std::sync::Arc;
use serde::Serialize;
use reqwest::StatusCode;
use crate::couchdb::responses::FindResponse;

pub mod name {
    pub const OAUTH: &'static str = "oauth";
}

pub struct Database {
    client: Arc<Client>,
    name: String,
    partitioned: bool,
}

impl Database {
    pub(super) fn new(client: Arc<Client>, name: String, partitioned: bool) -> Database {
        Database { client, name, partitioned, }
    }

    pub fn name(&self) -> &String {
        return &self.name;
    }

    pub async fn get_self(&self) -> reqwest::Result<responses::DBInfo> {
        log::debug!("getting info for database {}", self.name);
        self.client.get(self.name.as_str()).await
    }

    pub async fn create_self(&self) -> reqwest::Result<bool> {
        log::debug!("creating database {}", &self.name);
        self.client
            .put(self.name.as_str(), None::<bool>, Some(&[("partitioned", &self.partitioned)]))
            .await
            .map(|responses::Ok { ok }| ok)
    }

    pub async fn get<R: DeserializeOwned>(&self, id: &str) -> reqwest::Result<R> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("getting {} from couch", &path);
        self.client.get(path.as_str()).await
    }

    pub async fn put<T: Serialize, R: DeserializeOwned>(&self, id: &str, entity: T) -> reqwest::Result<R> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("putting {} into couch: {}", &path, serde_json::to_string(&entity).unwrap());
        self.client.put(path.as_str(), Some(entity), None::<usize>).await
    }

    pub async fn find<R: DeserializeOwned>(&self, selector: serde_json::Value) -> reqwest::Result<FindResponse<R>> {
        let path = format!("{}/_find", &self.name);
        let body = serde_json::json!({
            "selector": selector,
        });

        self.client.post(path.as_str(), Some(body), None::<bool>).await
    }

    pub async fn delete(&self, id: &str, rev: &str) -> reqwest::Result<()> {
        let path = format!("{}/{}", &self.name, id);
        self.client.delete(path.as_str(), Some(&[("rev", rev)])).await
    }

    pub async fn exists(&self, id: &str) -> reqwest::Result<bool> {
        let path = format!("{}/{}", &self.name, id);
        self.client.exists(path.as_str()).await
    }
}
