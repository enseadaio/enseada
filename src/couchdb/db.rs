use serde::de::DeserializeOwned;

use crate::couchdb::client::Client;
use crate::couchdb::responses;
use std::sync::Arc;

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

    pub async fn get<T: DeserializeOwned>(&self, id: &str) -> reqwest::Result<T> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("getting {} from couch", &path);
        self.client.get(path.as_str()).await
    }

    pub async fn create_self(&self) -> reqwest::Result<bool> {
        log::debug!("creating database {}", &self.name);
        self.client
            .put(self.name.as_str(), None::<bool>, Some(&[("partitioned", &self.partitioned)]))
            .await
            .map(|responses::Ok { ok }| ok)
    }
}
