use derivative::Derivative;
use reqwest::Client as HttpClient;
use url::{ParseError, Url};

use crate::couchdb::status::Status;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Client {
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
            .build().expect("HttpClient::build()");
        Client {
            client,
            base_url,
            username,
            password: Some(password),
        }
    }

    pub async fn status(&self) -> reqwest::Result<Status> {
        self.client.get(self.build_url("/_up").unwrap())
            .basic_auth(&self.username, self.password.as_ref())
            .send().await?
            .json::<Status>().await
    }

    fn build_url(&self, path: &str) -> Result<Url, ParseError> {
        self.base_url.join(path)
    }
}
