extern crate core;

use hyper::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

pub use changes::*;
pub use partition::Partition;

pub use crate::client::Client;
pub use crate::db::{AssertDatabaseParams, Database};
pub use crate::error::FutonError;
use crate::response::{Info, Up};

mod changes;
mod client;
mod db;
mod error;
mod partition;
pub mod request;
pub mod response;

pub type FutonResult<T> = Result<T, FutonError>;

#[derive(Debug, Clone)]
pub struct Couch {
    client: Client,
}

impl Couch {
    pub fn new<U: Into<Url>>(server_uri: U) -> Self {
        Self {
            client: Client::new(server_uri.into()),
        }
    }

    pub async fn info(&self) -> FutonResult<Info> {
        let req = self.client.request("/").method(Method::GET).body(())?;
        let (_, info) = self.client.execute(req).await?;
        Ok(info.unwrap())
    }

    pub async fn up(&self) -> FutonResult<Up> {
        let req = self.client.request("/_up").method(Method::GET).body(())?;
        let res = self.client.execute::<(), serde_json::Value>(req).await;
        if let Err(FutonError::Hyper(ref err)) = res {
            if err.is_connect() || err.is_closed() || err.is_timeout() {
                return Ok(Up::Unavailable);
            }
        }

        let up = res?.1.map(|_| Up::Ok).unwrap_or_else(|| Up::Unavailable);
        Ok(up)
    }

    pub fn database<N: ToString>(&self, name: N, partitioned: bool) -> Database {
        Database::new(name, partitioned, self.client.clone())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Envelope<T> {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    #[serde(rename = "_deleted", skip_serializing_if = "Option::is_none")]
    deleted: Option<bool>,
    #[serde(flatten)]
    item: T,
}

impl<T> Envelope<T> {
    pub fn new<I: ToString>(id: I, item: T) -> Self {
        Self {
            id: id.to_string(),
            rev: None,
            deleted: None,
            item,
        }
    }

    pub fn new_with_rev<I: ToString, R: ToString>(id: I, rev: R, item: T) -> Self {
        Self {
            id: id.to_string(),
            rev: Some(rev.to_string()),
            deleted: None,
            item,
        }
    }

    pub fn item(&self) -> &T {
        &self.item
    }

    pub fn item_mut(&mut self) -> &mut T {
        &mut self.item
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted.unwrap_or(false)
    }

    pub fn unwrap(self) -> T {
        self.item
    }

    pub fn unwrap_all(self) -> (String, Option<String>, T) {
        (self.id, self.rev, self.item)
    }
}
