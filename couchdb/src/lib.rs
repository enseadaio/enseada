#![forbid(unsafe_code)]

use std::sync::Arc;

use url::Url;

use crate::client::Client;
use crate::db::Database;
use crate::error::Error;
use crate::status::Status;

pub mod changes;
pub mod client;
pub mod db;
pub mod design_document;
/// couch db error
pub mod error;
pub mod index;
pub mod migrator;
pub mod responses;
pub mod status;
pub mod view;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Couch {
    client: Arc<Client>,
}

impl Couch {
    pub fn new(url: Url, username: String, password: String) -> Self {
        let client = Arc::new(Client::new(url, username, password));
        Couch { client }
    }

    pub fn database(&self, name: &str, partitioned: bool) -> Database {
        Database::new(self.client.clone(), name.to_string(), partitioned)
    }

    pub async fn status(&self) -> reqwest::Result<Status> {
        self.client.get("/_up", None::<bool>).await
    }
}
