use futures::StreamExt;

use hyper::Body;

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::changes::Changes;
use crate::partition::Partition;
use crate::response::RowsResponse;
use crate::{Client, Envelope, FutonResult, Method, StatusCode};
use crate::request::FindRequest;

#[derive(Debug, Clone)]
pub struct Database {
    name: String,
    path: String,
    client: Client,
    partitioned: bool,
}

impl Database {
    pub(crate) fn new<N: ToString>(name: N, partitioned: bool, client: Client) -> Self {
        let name = name.to_string();
        Self {
            path: format!("/{}", name),
            name,
            partitioned,
            client,
        }
    }

    pub fn partition<P: ToString>(&self, name: P) -> Partition {
        assert!(
            self.partitioned,
            "cannot call Database::partition on a non-partitioned database"
        );
        Partition::new(&self.name, name, self.client.clone())
    }

    #[tracing::instrument(skip(self))]
    pub async fn assert_self(&self, params: AssertDatabaseParams) -> FutonResult<()> {
        let req = self
            .client
            .request(&self.path)
            .method(Method::HEAD)
            .body(())?;
        if let (StatusCode::OK, _) = self.client.execute::<(), ()>(req).await? {
            return Ok(());
        }

        let qs = serde_qs::to_string(&params)?;
        let req = self
            .client
            .request(format!(
                "{}?partitioned={}&{}",
                self.path, self.partitioned, qs
            ))
            .method(Method::PUT)
            .body(())?;
        self.client.execute::<(), Value>(req).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list<T: DeserializeOwned + Debug>(
        &self,
        limit: usize,
        offset: usize,
    ) -> FutonResult<RowsResponse<Envelope<T>>> {
        let req = self
            .client
            .request(format!(
                "{}/_all_docs?limit={limit}&skip={offset}&include_docs=true",
                self.path
            ))
            .method(Method::GET)
            .body(())?;

        let (_, res) = self.client.execute(req).await?;
        Ok(res.expect("rows response should not be empty"))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get<Id: ToString + Debug, T: DeserializeOwned + Debug>(
        &self,
        id: Id,
    ) -> FutonResult<Option<Envelope<T>>> {
        let id = id.to_string();
        let req = self
            .client
            .request(format!("{}/{}", self.path, id))
            .method(Method::GET)
            .body(())?;
        let (_, obj) = self.client.execute(req).await?;
        Ok(obj)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_rev<
        Id: ToString + Debug,
        Rev: ToString + Debug,
        T: DeserializeOwned + Debug,
    >(
        &self,
        id: Id,
        rev: Rev,
    ) -> FutonResult<Option<Envelope<T>>> {
        let id = id.to_string();
        let rev = rev.to_string();
        let req = self
            .client
            .request(format!("{}/{id}?rev={rev}", self.path))
            .method(Method::GET)
            .body(())?;
        let (_, envelope) = self.client.execute(req).await?;
        Ok(envelope)
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by<S: Serialize + Debug, T: DeserializeOwned + Debug>(
        &self,
        selector: S,
    ) -> FutonResult<Option<Envelope<T>>> {
        let req = self
            .client
            .request(format!("{}/_find", self.path))
            .method(Method::POST)
            .body(FindRequest {
                selector,
                limit: 1,
                skip: 0,
            })?;
        let (_, envelope) = self.client.execute(req).await?;
        Ok(envelope)
    }

    #[tracing::instrument(skip(self))]
    pub async fn put<T: Serialize + Debug>(&self, envelope: Envelope<T>) -> FutonResult<()> {
        let req = self
            .client
            .request(&self.path)
            .method(Method::POST)
            .body(envelope)?;
        let ok = self.client.execute::<Envelope<T>, Value>(req).await?;
        tracing::debug!(?ok, "ok");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(
        &self,
        id: impl ToString + Debug,
        rev: impl ToString + Debug,
    ) -> FutonResult<()> {
        let req = self
            .client
            .request(format!(
                "{}/{}?rev={}",
                self.path,
                id.to_string(),
                rev.to_string()
            ))
            .method(Method::DELETE)
            .body(())?;
        self.client.execute::<(), ()>(req).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn bulk_insert<T: Serialize + Debug>(&self, docs: Vec<T>) -> FutonResult<()> {
        let req = self
            .client
            .request(format!("{}/_bulk_docs", self.path))
            .method(Method::POST)
            .body(serde_json::json!({
                "docs": docs,
            }))?;
        self.client.execute::<Value, Value>(req).await?;
        Ok(())
    }

    pub async fn changes(&self) -> FutonResult<Changes> {
        let client = self.client.clone();
        let path = format!(
            "{}/_changes?include_docs=true&feed=continuous&timeout=60000",
            self.path
        );
        let req = move |since| {
            client
                .clone()
                .request(format!("{path}&since={since}"))
                .method(Method::GET)
                .body(Body::empty())
                .unwrap()
        };

        Ok(Changes::new(self.client.clone(), req))
    }
}

#[derive(Debug, Serialize)]
pub struct AssertDatabaseParams {
    pub q: u32,
    pub n: u32,
}

impl Default for AssertDatabaseParams {
    fn default() -> Self {
        Self { q: 8, n: 3 }
    }
}
