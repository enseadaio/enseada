use std::fmt::Debug;

use headers::authorization::Basic;
use headers::{Authorization, ContentType, HeaderMapExt, UserAgent};
use hyper::client::HttpConnector;
use hyper::header::{ACCEPT, CONTENT_TYPE};
use hyper::http::response::Parts;
use hyper::{http, Body, Request, Response};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use crate::error::CouchError;
use crate::{FutonResult, StatusCode};

const USER_AGENT: &str = concat!("futon/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct Client {
    server_uri: Url,
    auth: Authorization<Basic>,
    inner: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Client {
    pub fn new(mut server_uri: Url) -> Self {
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        let inner = hyper::Client::builder().build(https);
        let username = server_uri.username();
        let password = server_uri.password().unwrap_or_default();
        let auth = Authorization::basic(username, password);
        server_uri.set_username("").unwrap();
        server_uri.set_password(None).unwrap();
        Self {
            server_uri,
            auth,
            inner,
        }
    }

    pub fn request(&self, path: impl ToString) -> http::request::Builder {
        let mut builder = Request::builder()
            .uri(format!(
                "{}{}",
                self.server_uri,
                path.to_string().trim_start_matches('/')
            ))
            .header(ACCEPT, "application/json");
        {
            let headers = builder.headers_mut().unwrap();
            headers.typed_insert(self.auth.clone());
            headers.typed_insert(UserAgent::from_static(USER_AGENT));
        }
        builder
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute<T: Serialize + Debug, B: DeserializeOwned + Debug>(
        &self,
        req: Request<T>,
    ) -> FutonResult<(StatusCode, Option<B>)> {
        let (mut parts, body) = req.into_parts();
        parts.headers.typed_insert(ContentType::json());
        let body = serde_json::to_vec(&body)?;
        let req = Request::from_parts(parts, Body::from(body));
        let res = self.raw_execute(req).await?;
        tracing::debug!(?res, "executed CouchDB request");
        let (parts, body) = parse_couch_response(res).await?;
        Ok((parts.status, body))
    }

    pub async fn raw_execute(&self, req: Request<Body>) -> hyper::Result<Response<Body>> {
        self.inner.request(req).await
    }
}

async fn parse_couch_response<T: DeserializeOwned>(
    res: Response<Body>,
) -> Result<(Parts, Option<T>), CouchError> {
    let (parts, body) = res.into_parts();
    match parts.status {
        StatusCode::OK | StatusCode::CREATED => {
            let bytes = hyper::body::to_bytes(body).await?;
            if bytes.is_empty() {
                return Ok((parts, None));
            }
            match parts.headers.get(CONTENT_TYPE).expect("").as_bytes() {
                b"application/json" => Ok((parts, Some(serde_json::from_slice(&bytes)?))),
                ct => panic!("invalid content type: '{:?}'", ct),
            }
        }
        StatusCode::NOT_FOUND => Ok((parts, None)),
        _ => {
            let bytes = hyper::body::to_bytes(body).await?;
            Err(CouchError::from((parts, bytes)))
        }
    }
}
