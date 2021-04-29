use bytes::Bytes;
use derivative::Derivative;
use reqwest::{Client as HttpClient, Method, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use url::{ParseError, Url};

use crate::responses::Ok;
use reqwest::header::ACCEPT;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub(super) struct Client {
    client: HttpClient,
    base_url: Url,
    username: String,
    #[derivative(Debug = "ignore")]
    password: Option<String>,
}

impl Client {
    pub fn new(base_url: Url, username: String, password: String) -> Client {
        let client = HttpClient::builder()
            .use_rustls_tls()
            .build()
            .expect("HttpClient::build()");
        Client {
            client,
            base_url,
            username,
            password: Some(password),
        }
    }

    pub async fn get<Q: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<Q>,
    ) -> reqwest::Result<T> {
        self.request(Method::GET, path, None::<bool>, query).await
    }

    pub async fn put<B: Serialize, Q: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: Option<B>,
        query: Option<Q>,
    ) -> reqwest::Result<R> {
        self.request(Method::PUT, path, body, query).await
    }

    pub async fn post<B: Serialize, Q: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: Option<B>,
        query: Option<Q>,
    ) -> reqwest::Result<R> {
        self.request(Method::POST, path, body, query).await
    }

    pub async fn delete<Q: Serialize>(&self, path: &str, query: Option<Q>) -> reqwest::Result<()> {
        self.request(Method::DELETE, path, None::<bool>, query)
            .await
            .map(|_: Ok| ())
    }

    pub async fn exists(&self, path: &str) -> reqwest::Result<bool> {
        let result = self
            .client
            .head(self.build_url(path).unwrap())
            .basic_auth(&self.username, self.password.as_ref())
            .send()
            .await?
            .error_for_status();

        match result {
            Ok(_res) => Ok(true),
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Ok(false),
                _ => Err(err),
            },
        }
    }

    pub async fn stream<B: Serialize, Q: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<B>,
        query: Option<Q>,
    ) -> reqwest::Result<impl futures::Stream<Item = reqwest::Result<Bytes>>> {
        let req = self.build_req(method, path);
        let req = if let Some(body) = body {
            req.json::<B>(&body)
        } else {
            req
        };
        let req = if let Some(query) = query {
            req.query::<Q>(&query)
        } else {
            req
        };
        let res = req.send().await?.error_for_status()?;
        Ok(res.bytes_stream())
    }

    pub(crate) fn build_url(&self, path: &str) -> Result<Url, ParseError> {
        self.base_url.join(path)
    }

    pub async fn request<B: Serialize, Q: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<B>,
        query: Option<Q>,
    ) -> reqwest::Result<R> {
        let req = self.build_req(method, path);
        let req = if let Some(body) = body {
            req.json::<B>(&body)
        } else {
            req
        };

        let req = if let Some(query) = query {
            req.query::<Q>(&query)
        } else {
            req
        };
        let req = req.header(ACCEPT, "application/json");

        req.send().await?.error_for_status()?.json().await
    }

    fn build_req(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, self.build_url(path).unwrap())
            .basic_auth(&self.username, self.password.as_ref())
    }
}
