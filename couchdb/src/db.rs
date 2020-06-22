use std::fmt::{self, Debug, Formatter};
use std::io::{BufRead, Cursor};
use std::str::from_utf8;
use std::sync::Arc;

use bytes::Bytes;
use futures::{stream, StreamExt};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::changes::ChangeEvent;
use crate::client::Client;
use crate::error::Error;
use crate::index::JsonIndex;
use crate::responses;
use crate::responses::{
    FindResponse, JsonIndexResponse, JsonIndexResultStatus, Partition, PutResponse, RowsResponse,
};
use crate::Result;

#[derive(Clone)]
pub struct Database {
    client: Arc<Client>,
    name: String,
    partitioned: bool,
}

impl Database {
    pub(super) fn new(client: Arc<Client>, name: String, partitioned: bool) -> Database {
        Database {
            client,
            name,
            partitioned,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub async fn get_self(&self) -> Result<responses::DBInfo> {
        log::debug!("Getting info for database {}", self.name);
        self.client
            .get(self.name.as_str(), None::<bool>)
            .await
            .map_err(Error::from)
    }

    pub async fn create_self(&self) -> Result<bool> {
        log::debug!("Creating database {}", &self.name);
        let res: responses::Ok = self
            .client
            .put(
                self.name.as_str(),
                None::<bool>,
                Some(&[("partitioned", &self.partitioned)]),
            )
            .await?;
        Ok(res.ok)
    }

    pub async fn create_index(&self, index: JsonIndex) -> Result<bool> {
        let path = format!("{}/_index", &self.name);
        log::debug!(
            "Creating index {} on database {}",
            &index.name(),
            &self.name
        );

        let res: JsonIndexResponse = self.client.post(&path, Some(index), None::<bool>).await?;
        match res.result {
            JsonIndexResultStatus::Created => Ok(true),
            JsonIndexResultStatus::Exists => Ok(false),
        }
    }

    pub async fn count_partitioned(&self, partition: &str) -> Result<usize> {
        log::debug!(
            "counting all docs in partition {} of {}",
            partition,
            &self.name
        );
        let path = format!("{}/_partition/{}", &self.name, partition);
        let part = self
            .client
            .get::<bool, Partition>(&path, None::<bool>)
            .await?;
        Ok(part.doc_count)
    }

    pub async fn count(&self) -> Result<usize> {
        log::debug!("counting all docs in {}", &self.name);
        let db = self.get_self().await?;
        Ok(db.doc_count)
    }

    pub async fn get<R: DeserializeOwned>(&self, id: &str) -> Result<Option<R>> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Getting {} from couch", &path);
        match self.client.get(&path, None::<bool>).await {
            Ok(r) => Ok(Some(r)),
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Ok(None),
                _ => Err(Error::from(err)),
            },
        }
    }

    pub async fn list_partitioned<R: DeserializeOwned + Clone>(
        &self,
        partition: &str,
        limit: usize,
        start_key: Option<String>,
    ) -> Result<RowsResponse<R>> {
        let path = format!("{}/_partition/{}/_all_docs", &self.name, partition);
        self.do_list(&path, limit, start_key).await
    }

    pub async fn list<R: DeserializeOwned + Clone>(
        &self,
        limit: usize,
        start_key: Option<String>,
    ) -> Result<RowsResponse<R>> {
        let path = format!("{}/_all_docs", &self.name);
        self.do_list(&path, limit, start_key).await
    }

    async fn do_list<R: DeserializeOwned + Clone>(
        &self,
        path: &str,
        limit: usize,
        start_key: Option<String>,
    ) -> Result<RowsResponse<R>> {
        let query = Some(ListQuery {
            include_docs: true,
            limit,
            start_key,
        });
        self.client.get(path, query).await.map_err(Error::from)
    }

    pub async fn list_all_partitioned<R: DeserializeOwned + Clone>(
        &self,
        partition: &str,
    ) -> Result<RowsResponse<R>> {
        let path = format!("{}/_partition/{}/_all_docs", &self.name, partition);
        self.client
            .get(&path, Some(&[("include_docs", true)]))
            .await
            .map_err(Error::from)
    }

    pub async fn put<T: Serialize>(&self, id: &str, entity: T) -> Result<PutResponse> {
        let path = format!("{}/{}", &self.name, &id);
        log::debug!("Putting {} into couch", &path);
        self.client
            .put(&path, Some(entity), None::<usize>)
            .await
            .map_err(|err| match err.status() {
                Some(StatusCode::CONFLICT) => Error::map_message(
                    err,
                    &format!("document {} already exists in database {}", &id, &self.name),
                ),
                _ => Error::from(err),
            })
    }

    pub async fn find<R: DeserializeOwned>(
        &self,
        selector: serde_json::Value,
        limit: usize,
        bookmark: Option<String>,
    ) -> Result<FindResponse<R>> {
        let path = format!("{}/_find", &self.name);
        self.do_find(&path, selector, limit, bookmark).await
    }

    pub async fn find_partitioned<R: DeserializeOwned>(
        &self,
        partition: &str,
        selector: serde_json::Value,
        limit: usize,
        bookmark: Option<String>,
    ) -> Result<FindResponse<R>> {
        let path = format!("{}/_partition/{}/_find", &self.name, partition);
        self.do_find(&path, selector, limit, bookmark).await
    }

    async fn do_find<R: DeserializeOwned>(
        &self,
        path: &str,
        selector: serde_json::Value,
        limit: usize,
        bookmark: Option<String>,
    ) -> Result<FindResponse<R>> {
        let body = serde_json::json!({
            "selector": selector,
            "limit": limit,
            "bookmark": bookmark
        });

        log::debug!("Finding from {} with query {}", &self.name, &body);

        self.client
            .post(path, Some(body), None::<bool>)
            .await
            .map_err(Error::from)
    }

    pub async fn delete(&self, id: &str, rev: &str) -> Result<()> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Deleting {} from couch", &path);
        self.client.delete(&path, Some(&[("rev", rev)])).await?;
        Ok(())
    }

    pub async fn exists(&self, id: &str) -> Result<bool> {
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Checking {} existence from couch", &path);
        let res = self.client.exists(&path).await?;
        Ok(res)
    }

    pub async fn changes(&self) -> Result<impl futures::Stream<Item = ChangeEvent>> {
        let path = format!("{}/_changes", &self.name);
        let query = Some(&[("feed", "continuous"), ("since", "now")]);
        let stream = self.client.stream(&path, query).await?;
        let stream = stream
            .map(|res: reqwest::Result<Bytes>| {
                let bytes = res.unwrap();
                let payload = from_utf8(bytes.as_ref()).unwrap();
                let cursor = Cursor::new(payload);
                let events: Vec<ChangeEvent> = cursor
                    .lines()
                    .filter_map(|line| match line {
                        Ok(line) => {
                            if line.is_empty() {
                                None
                            } else {
                                log::trace!("Processing event: {}", &line);
                                let event: ChangeEvent = serde_json::from_str(&line).unwrap();
                                Some(event)
                            }
                        }
                        Err(err) => {
                            log::error!("{}", err.to_string());
                            None
                        }
                    })
                    .collect();
                stream::iter(events)
            })
            .flatten();
        Ok(stream)
    }
}

impl Debug for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Database")
            .field("name", &self.name)
            .field("partitioned", &self.partitioned)
            .finish()
    }
}

#[derive(Serialize)]
struct ListQuery {
    pub include_docs: bool,
    pub limit: usize,
    pub start_key: Option<String>,
}
