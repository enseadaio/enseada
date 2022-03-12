use hyper::{Method, StatusCode};
use std::borrow::Cow;

pub use crate::client::Client;
use crate::error::CouchError;
pub use crate::error::FutonError;
use crate::model::{Info, Up};

mod client;
mod error;
pub mod model;

pub type CouchResult<T> = Result<T, FutonError>;

#[derive(Debug, Clone)]
pub struct Couch {
    client: Client,
}

impl Couch {
    pub fn new<U: Into<Cow<'static, str>>>(server_uri: U) -> Self {
        Self {
            client: Client::new(server_uri.into()),
        }
    }

    pub async fn info(&self) -> CouchResult<Info> {
        let req = self.client.request("/").method(Method::GET).body(())?;
        let (_, body) = self.client.execute(req).await?;
        Ok(body.unwrap())
    }

    pub async fn up(&self) -> CouchResult<Up> {
        let req = self.client.request("/_up").method(Method::GET).body(())?;
        let res = self.client.execute::<(), serde_json::Value>(req).await;
        if let Err(FutonError::Hyper(ref err)) = res {
            if err.is_connect() || err.is_closed() || err.is_timeout() {
                return Ok(Up::Unavailable);
            }
        }

        let (parts, _) = res?;
        let up = match parts.status {
            StatusCode::OK => Up::Ok,
            StatusCode::NOT_FOUND => Up::Unavailable,
            _ => return Err(CouchError::Internal.into()),
        };

        Ok(up)
    }
}
