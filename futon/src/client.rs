use crate::error::CouchError;
use crate::CouchResult;
use hyper::client::HttpConnector;
use hyper::http::response::Parts;
use hyper::{http, Body, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Cow;

const USER_AGENT: &str = concat!("futon/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct Client {
    server_uri: Cow<'static, str>,
    inner: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Client {
    pub fn new(server_uri: Cow<'static, str>) -> Self {
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        let inner = hyper::Client::builder().build(https);
        Self { server_uri, inner }
    }

    pub fn request(&self, path: &str) -> http::request::Builder {
        Request::builder()
            .uri(format!("{}{}", self.server_uri, path))
            .header(http::header::ACCEPT, "application/json")
            .header(http::header::USER_AGENT, USER_AGENT)
    }

    pub async fn execute<T: Serialize, B: DeserializeOwned>(
        &self,
        req: Request<T>,
    ) -> CouchResult<(Parts, Option<B>)> {
        let (parts, body) = req.into_parts();
        let body = serde_json::to_vec(&body)?;
        let req = Request::from_parts(parts, Body::from(body));
        let res = self.inner.request(req).await?;
        if res.status().is_client_error() || res.status().is_server_error() {
            return Err(CouchError::from(res).into());
        }
        let (parts, body) = res.into_parts();
        let bytes = hyper::body::to_bytes(body).await?;
        Ok((parts, Some(serde_json::from_slice(&bytes)?)))
    }
}
