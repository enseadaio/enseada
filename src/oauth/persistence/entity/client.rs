use serde::{Deserialize, Serialize};
use crate::oauth::client::Client;
use crate::couchdb::guid::Guid;
use crate::oauth::Scope;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientEntity {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    allowed_scopes: Scope,
    allowed_redirect_uris: Vec<String>,
}

impl ClientEntity {
    pub fn build_guid(client_id: &String) -> Guid {
        Guid::from(format!("client:{}", client_id))
    }
}

impl From<Client> for ClientEntity {
    fn from(client: Client) -> Self {
        let id = Self::build_guid(client.client_id());
        let uris = client.allowed_redirect_uris().clone();
        let allowed_redirect_uris = uris.iter().map(|url| url.to_string()).collect();
        ClientEntity {
            id,
            rev: None,
            allowed_scopes: client.allowed_scopes().clone(),
            allowed_redirect_uris,
        }
    }
}

impl Into<Client> for ClientEntity {
    fn into(self) -> Client {
        let guid = &self.id;
        let uris = self.allowed_redirect_uris.clone();
        let allowed_redirect_uris = uris.iter().map(|url| Url::parse(url).unwrap()).collect(); // TODO: handle
        Client::public(guid.id().clone(),
                       self.allowed_scopes.clone(),
                       allowed_redirect_uris)
    }
}