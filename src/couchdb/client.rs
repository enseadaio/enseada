use derivative::Derivative;
use reqwest::{Client as HttpClient, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use url::{ParseError, Url};




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
        log::debug!("Creating new CouchDB client instance");
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

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> reqwest::Result<T> {
        self.client
            .get(self.build_url(path).unwrap())
            .basic_auth(&self.username, self.password.as_ref())
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await
    }

    pub async fn put<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: Option<B>,
    ) -> reqwest::Result<R> {
        let req = self
            .client
            .put(self.build_url(path).unwrap())
            .basic_auth(&self.username, self.password.as_ref());
        let req = if let Some(body) = body {
            req.json::<B>(&body)
        } else {
            req
        };

        req.send().await?.error_for_status()?.json().await
    }

    pub async fn exists(&self, path: &str) -> Result<bool, reqwest::Error> {
        let result = self
            .client
            .head(self.build_url(path).unwrap())
            .basic_auth(&self.username, self.password.as_ref())
            .send()
            .await;

        match result {
            Ok(_res) => Ok(true),
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Ok(false),
                Some(_) | None => Err(err),
            },
        }
    }

    pub(crate) fn build_url(&self, path: &str) -> Result<Url, ParseError> {
        self.base_url.join(path)
    }
}
