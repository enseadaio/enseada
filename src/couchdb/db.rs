use std::fmt::Debug;
use std::io::{BufRead, BufReader, Cursor};
use std::str::from_utf8;
use std::sync::Arc;

use bytes::Bytes;
use futures::{stream, StreamExt};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::couchdb::changes::ChangeEvent;
use crate::couchdb::client::Client;
use crate::couchdb::error::Error;
use crate::couchdb::index::JsonIndex;
use crate::couchdb::responses;
use crate::couchdb::responses::{FindResponse, JsonIndexResponse, JsonIndexResultStatus, PutResponse, RowsResponse};
use crate::couchdb::Result;

pub mod name {
    pub const OAUTH: &str = "oauth";
    pub const USERS: &str = "users";
    pub const RBAC: &str = "rbac";
}

#[derive(Clone)]
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

    pub async fn create_index(&self, index: JsonIndex) -> Result<bool> {
        let path = format!("{}/_index", &self.name);
        log::debug!("Creating index {} on database {}", &index.name(), &self.name);

        let res: JsonIndexResponse = self.client.post(&path, Some(index), None::<bool>).await?;
        match res.result {
            JsonIndexResultStatus::Created => Ok(true),
            JsonIndexResultStatus::Exists => Ok(false),
        }
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

    pub async fn list<R: DeserializeOwned + Clone>(&self, kind: &str, limit: usize, start_key: Option<String>) -> Result<RowsResponse<R>> {
        let path = format!("{}/_partition/{}/_all_docs", &self.name, kind);
        let query = Some(ListQuery {
            include_docs: true,
            limit,
            start_key,
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

    pub async fn find<R: DeserializeOwned>(&self, selector: serde_json::Value, limit: usize, bookmark: Option<String>) -> Result<FindResponse<R>> {
        let path = format!("{}/_find", &self.name);
        self.do_find(&path, selector, limit, bookmark).await
    }

    pub async fn find_partitioned<R: DeserializeOwned>(&self, partition: &str, selector: serde_json::Value, limit: usize, bookmark: Option<String>) -> Result<FindResponse<R>> {
        let path = format!("{}/_partition/{}/_find", &self.name, partition);
        self.do_find(&path, selector, limit, bookmark).await
    }

    async fn do_find<R: DeserializeOwned>(&self, path: &str, selector: serde_json::Value, limit: usize, bookmark: Option<String>) -> Result<FindResponse<R>> {
        let body = serde_json::json!({
            "selector": selector,
            "limit": limit,
            "bookmark": bookmark
        });

        log::debug!("Finding from {} with query {}", &self.name, &body);

        self.client.post(path, Some(body), None::<bool>).await
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

    pub async fn changes(&self) -> Result<impl futures::Stream<Item=ChangeEvent>> {
        let path = format!("{}/_changes", &self.name);
        let query = Some(&[("feed", "continuous"), ("since", "now")]);
        let stream = self.client.stream(&path, query).await?;
        let stream = stream.map(|res: reqwest::Result<Bytes>| {
            let bytes = res.unwrap();
            let payload = from_utf8(bytes.as_ref()).unwrap();
            let cursor = Cursor::new(payload);
            let events: Vec<ChangeEvent> = cursor.lines().filter_map(|line| {
                match line {
                    Ok(line) => if line.is_empty() {
                        None
                    } else {
                        log::debug!("Processing event: {}", &line);
                        let event: ChangeEvent = serde_json::from_str(&line).unwrap();
                        Some(event)
                    },
                    Err(err) => {
                        log::error!("{}", err.to_string());
                        None
                    }
                }
            }).collect();
            stream::iter(events)
        }).flatten();
        Ok(stream)
    }
}

#[derive(Serialize)]
struct ListQuery {
    pub include_docs: bool,
    pub limit: usize,
    pub start_key: Option<String>,
}
