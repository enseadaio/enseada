use crate::oauth::{Scope};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Client {
    client_id: String,
    client_secret: Option<String>,
    allowed_scopes: Scope,
    allowed_redirect_uris: HashSet<url::Url>,
}

impl Client {
    pub fn confidential(client_id: String,
                        client_secret: String,
                        allowed_scopes: Scope,
                        allowed_redirect_uris: HashSet<url::Url>) -> Client {
        Client {
            client_id,
            client_secret: Some(client_secret),
            allowed_scopes,
            allowed_redirect_uris
        }
    }

    pub fn public(client_id: String,
                  allowed_scopes: Scope,
                  allowed_redirect_uris: HashSet<url::Url>) -> Client {
        Client {
            client_id,
            client_secret: None,
            allowed_scopes,
            allowed_redirect_uris,
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

    pub fn allowed_redirect_uris(&self) -> &HashSet<url::Url> {
        &self.allowed_redirect_uris
    }

}
