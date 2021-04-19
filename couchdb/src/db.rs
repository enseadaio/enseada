use std::fmt::{self, Debug, Formatter};
use std::io::{BufRead, Cursor};
use std::str::from_utf8;
use std::sync::Arc;

use bytes::Bytes;
use futures::{stream, Stream, StreamExt};
use percent_encoding::{AsciiSet, CONTROLS, percent_encode};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::changes::{ChangeEvent, ChangeRequest};
use crate::client::Client;
use crate::design_document::{DesignDocument, ViewDoc};
use crate::error::Error;
use crate::index::JsonIndex;
use crate::responses;
use crate::responses::{FindResponse, JsonIndexResponse, JsonIndexResultStatus, OkWrapper, Partition, PutResponse, RevisionList, Revs, RowsResponse};
use crate::Result;
use crate::view::View;

const ESCAPED: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'+')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');

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

    pub fn view<D: ToString, N: ToString>(&self, ddoc: D, name: N) -> View {
        View::new(
            self.client.clone(),
            self.name.clone(),
            ddoc.to_string(),
            name.to_string(),
        )
    }

    pub fn partitioned_view<D: ToString, P: ToString, N: ToString>(
        &self,
        ddoc: D,
        partition: P,
        name: N,
    ) -> View {
        View::partitioned(
            self.client.clone(),
            self.name.clone(),
            ddoc.to_string(),
            partition.to_string(),
            name.to_string(),
        )
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

    pub async fn create_view(&self, ddoc: &str, view: ViewDoc) -> Result<()> {
        let id = format!("_design/{}", ddoc);

        let mut design_doc = self
            .find_one::<DesignDocument>(&id)
            .await?
            .unwrap_or_else(|| DesignDocument::new(ddoc, true));
        design_doc.add_view(view);

        let path = format!("{}/{}", &self.name, id);
        self.client
            .put::<DesignDocument, bool, PutResponse>(&path, Some(design_doc), None::<bool>)
            .await?;
        Ok(())
    }

    pub async fn count_partitioned(&self, partition: &str) -> Result<usize> {
        log::debug!(
            "counting all docs in partition {} of {}",
            partition,
            &self.name
        );
        let partition = percent_encode(partition.as_bytes(), ESCAPED).to_string();
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

    pub async fn get<R: DeserializeOwned>(&self, id: &str) -> Result<R> {
        let id = percent_encode(id.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Getting {} from couch", &path);
        self.client.get(&path, None::<bool>).await.map_err(Error::from)
    }

    pub async fn get_at<R: DeserializeOwned>(&self, id: &str, rev: &str) -> Result<R> {
        let id = percent_encode(id.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Getting {} from couch with rev {}", &path, rev);
        self.client.get(&path, Some(&[("rev", rev)])).await.map_err(Error::from)
    }

    pub async fn get_revs(&self, id: &str) -> Result<Revs> {
        let id = percent_encode(id.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Getting revs for {} from couch", &path);
        let results: Vec<OkWrapper<Revs>> = self.client.get(&path, Some(&[("revs", "true"), ("open_revs", "all")])).await.map_err(Error::from)?;
        match results.into_iter().next() {
            Some(OkWrapper { ok: revs, }) => Ok(revs),
            None => Err(Error::not_found(format!("Could not find any revs for {}", path))),
        }
    }

    pub async fn find_one<R: DeserializeOwned>(&self, id: &str) -> Result<Option<R>> {
        let id = percent_encode(id.as_bytes(), ESCAPED).to_string();
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

    pub async fn list_partitioned<R: DeserializeOwned>(
        &self,
        partition: &str,
        limit: usize,
        skip: usize,
    ) -> Result<RowsResponse<R>> {
        let partition = percent_encode(partition.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/_partition/{}/_all_docs", &self.name, partition);
        self.do_list(&path, limit, skip).await
    }

    pub async fn list<R: DeserializeOwned>(
        &self,
        limit: usize,
        skip: usize,
    ) -> Result<RowsResponse<R>> {
        let path = format!("{}/_all_docs", &self.name);
        self.do_list(&path, limit, skip).await
    }

    pub fn stream<R: DeserializeOwned>(&self) -> impl Stream<Item = Result<R>> {
        let path = format!("{}/_all_docs", &self.name);
        stream::try_unfold(
            (0, path, self.client.clone()),
            |(skip, path, client)| async move {
                let query = Some(ListQuery {
                    include_docs: true,
                    limit: 100,
                    skip,
                });
                match client.get::<ListQuery, RowsResponse<R>>(&path, query).await {
                    Ok(res) => {
                        if res.rows.is_empty() {
                            Ok(None)
                        } else {
                            let rows = res
                                .rows
                                .into_iter()
                                .map(|raw| raw.doc.unwrap())
                                .collect::<Vec<R>>();
                            Ok(Some((rows, (skip + 100, path, client))))
                        }
                    }
                    Err(err) => Err(Error::from(err)),
                }
            },
        )
        .map(|res| match res {
            Ok(iter) => iter.into_iter().map(Result::Ok).collect(),
            Err(err) => vec![Err(err)],
        })
        .map(stream::iter)
        .flatten()
    }

    async fn do_list<R: DeserializeOwned>(
        &self,
        path: &str,
        limit: usize,
        skip: usize,
    ) -> Result<RowsResponse<R>> {
        let query = Some(ListQuery {
            include_docs: true,
            limit,
            skip,
        });
        self.client.get(path, query).await.map_err(Error::from)
    }

    pub async fn list_all_partitioned<R: DeserializeOwned>(
        &self,
        partition: &str,
    ) -> Result<RowsResponse<R>> {
        let partition = percent_encode(partition.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/_partition/{}/_all_docs", &self.name, partition);
        self.client
            .get(&path, Some(&[("include_docs", true)]))
            .await
            .map_err(Error::from)
    }

    pub async fn put<T: Serialize, ID: AsRef<[u8]>>(
        &self,
        id: ID,
        entity: T,
    ) -> Result<PutResponse> {
        let id = percent_encode(id.as_ref(), ESCAPED).to_string();
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
        skip: usize,
    ) -> Result<FindResponse<R>> {
        let path = format!("{}/_find", &self.name);
        self.do_find(&path, selector, limit, skip).await
    }

    pub async fn find_partitioned<R: DeserializeOwned>(
        &self,
        partition: &str,
        selector: serde_json::Value,
        limit: usize,
        skip: usize,
    ) -> Result<FindResponse<R>> {
        let partition = percent_encode(partition.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/_partition/{}/_find", &self.name, partition);
        self.do_find(&path, selector, limit, skip).await
    }

    async fn do_find<R: DeserializeOwned>(
        &self,
        path: &str,
        selector: serde_json::Value,
        limit: usize,
        skip: usize,
    ) -> Result<FindResponse<R>> {
        let body = serde_json::json!({
            "selector": selector,
            "limit": limit,
            "skip": skip,
        });

        log::debug!("Finding from {} with query {}", &self.name, &body);

        self.client
            .post(path, Some(body), None::<bool>)
            .await
            .map_err(Error::from)
    }

    pub fn find_stream<R: DeserializeOwned>(
        &self,
        selector: serde_json::Value,
    ) -> impl Stream<Item = Result<R>> {
        let path = format!("{}/_find", &self.name);
        self.do_find_stream(path, selector)
    }

    pub fn find_partitioned_stream<R: DeserializeOwned>(
        &self,
        partition: &str,
        selector: serde_json::Value,
    ) -> impl Stream<Item = Result<R>> {
        let partition = percent_encode(partition.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/_partition/{}/_find", &self.name, partition);
        self.do_find_stream(path, selector)
    }

    fn do_find_stream<R: DeserializeOwned>(
        &self,
        path: String,
        selector: serde_json::Value,
    ) -> impl Stream<Item = Result<R>> {
        stream::try_unfold(
            (0, path, self.client.clone(), selector),
            |(skip, path, client, selector)| async move {
                let body = serde_json::json!({
                    "selector": &selector,
                    "limit": 100,
                    "skip": skip,
                });
                match client
                    .post::<serde_json::Value, bool, FindResponse<R>>(
                        &path,
                        Some(body),
                        None::<bool>,
                    )
                    .await
                {
                    Ok(res) => {
                        if let Some(warning) = &res.warning {
                            log::warn!("{}", warning);
                        }

                        if res.docs.is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some((res.docs, (skip + 100, path, client, selector))))
                        }
                    }
                    Err(err) => Err(Error::from(err)),
                }
            },
        )
        .map(|res| match res {
            Ok(iter) => iter.into_iter().map(Result::Ok).collect(),
            Err(err) => vec![Err(err)],
        })
        .map(stream::iter)
        .flatten()
    }

    pub async fn delete(&self, id: &str, rev: &str) -> Result<()> {
        let id = percent_encode(id.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Deleting {} from couch", &path);
        self.client.delete(&path, Some(&[("rev", rev)])).await?;
        Ok(())
    }

    pub async fn exists(&self, id: &str) -> Result<bool> {
        let id = percent_encode(id.as_bytes(), ESCAPED).to_string();
        let path = format!("{}/{}", &self.name, id);
        log::debug!("Checking {} existence from couch", &path);
        let res = self.client.exists(&path).await?;
        Ok(res)
    }

    pub async fn changes_since(&self, seq: String) -> Result<impl futures::Stream<Item = ChangeEvent>> {
        let path = format!("{}/_changes", &self.name);
        let query = Some(ChangeRequest {
            feed: "continuous".to_string(),
            since: seq,
        });
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

    pub async fn changes(&self) -> Result<impl futures::Stream<Item = ChangeEvent>> {
        self.changes_since("now".to_string()).await
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
pub(super) struct ListQuery {
    pub include_docs: bool,
    pub limit: usize,
    pub skip: usize,
}
