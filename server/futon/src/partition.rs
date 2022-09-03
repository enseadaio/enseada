use crate::response::RowsResponse;
use crate::{Client, Envelope, FutonResult, Method};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Partition {
    db: String,
    name: String,
    client: Client,
}

impl Partition {
    pub(crate) fn new<D: ToString, N: ToString>(db: D, name: N, client: Client) -> Self {
        let db = db.to_string();
        let name = name.to_string();
        Self { db, name, client }
    }

    pub async fn all_docs<T: DeserializeOwned + Debug>(
        &self,
    ) -> FutonResult<RowsResponse<Envelope<T>>> {
        let req = self
            .client
            .request(format!(
                "/{}/_partition/{}/_all_docs?include_docs=true",
                self.db, self.name
            ))
            .method(Method::GET)
            .body(())?;

        let (_, res) = self.client.execute(req).await?;
        Ok(res.expect("rows response should not be empty"))
    }
}
