use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{self, Debug, Formatter};

use crate::client::Client;
use crate::db::ListQuery;
use crate::error::Error;
use crate::responses::RowsResponse;
use crate::Result;

pub struct View {
    client: Arc<Client>,
    db_name: String,
    ddoc: String,
    name: String,
    partion: Option<String>,
}

impl View {
    pub(super) fn new(client: Arc<Client>, db_name: String, ddoc: String, name: String) -> Self {
        Self {
            client,
            db_name,
            ddoc,
            name,
            partion: None,
        }
    }

    pub(super) fn partitioned(
        client: Arc<Client>,
        db_name: String,
        ddoc: String,
        partition: String,
        name: String,
    ) -> Self {
        Self {
            client,
            db_name,
            ddoc,
            name,
            partion: Some(partition),
        }
    }

    pub async fn list_all_for_key<K: Serialize, T: DeserializeOwned>(
        &self,
        key: K,
        include_docs: bool,
    ) -> Result<RowsResponse<T>> {
        let partition = self
            .partion
            .as_ref()
            .map(|p| format!("/_partition/{}/", p))
            .unwrap_or_else(|| "/".to_string());
        let path = format!(
            "{}{}_design/{}/_view/{}",
            self.db_name, partition, self.ddoc, self.name
        );
        let body = ViewQueryAll {
            key: Some(serde_json::to_value(key).unwrap()),
            include_docs,
        };
        self.client
            .post(&path, Some(body), None::<bool>)
            .await
            .map_err(Error::from)
    }

    pub async fn list_for_key<K: Serialize, T: DeserializeOwned>(
        &self,
        key: K,
        limit: usize,
        skip: usize,
        include_docs: bool,
    ) -> Result<RowsResponse<T>> {
        let partition = self
            .partion
            .as_ref()
            .map(|p| format!("/_partition/{}/", p))
            .unwrap_or_else(|| "/".to_string());
        let path = format!(
            "{}{}_design/{}/_view/{}",
            self.db_name, partition, self.ddoc, self.name
        );
        let body = ViewQueryPage {
            key: Some(serde_json::to_value(key).unwrap()),
            list: ListQuery {
                include_docs,
                limit,
                skip,
            },
        };
        self.client
            .post(&path, Some(body), None::<bool>)
            .await
            .map_err(Error::from)
    }
}

impl Debug for View {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("View")
            .field("db_name", &self.db_name)
            .field("ddoc", &self.ddoc)
            .field("name", &self.name)
            .field("partion", &self.partion)
            .finish()
    }
}

#[derive(Serialize)]
struct ViewQueryPage {
    pub key: Option<serde_json::Value>,
    #[serde(flatten)]
    pub list: ListQuery,
}

#[derive(Serialize)]
struct ViewQueryAll {
    pub key: Option<serde_json::Value>,
    pub include_docs: bool,
}
