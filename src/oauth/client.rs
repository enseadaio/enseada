use crate::oauth::{Scope};

#[derive(Clone)]
pub struct Client {
    client_id: String,
    client_secret: Option<String>,
    allowed_scopes: Scope,
    allowed_redirect_uris: Vec<url::Url>,
}

impl Client {
    pub fn new(client_id: String,
               client_secret: Option<String>,
               allowed_scopes: Scope,
               allowed_redirect_uris: Vec<url::Url>) -> Client{
        Client {
            client_id,
            client_secret,
            allowed_scopes,
            allowed_redirect_uris
        }
    }

    pub fn client_id(&self) -> &String {
        &self.client_id
    }

    pub fn client_secret(&self) -> Option<String> {
        self.client_secret.clone()
    }

    pub fn allowed_scopes(&self) -> &Scope {
        &self.allowed_scopes
    }

    pub fn allowed_redirect_uris(&self) -> &Vec<url::Url> {
        &self.allowed_redirect_uris
    }

}
