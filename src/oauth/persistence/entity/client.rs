use serde::{Deserialize, Serialize};
use crate::oauth::client::Client;
use crate::couchdb::guid::Guid;
use crate::oauth::Scope;

#[derive(Deserialize, Serialize)]
pub struct ClientEntity {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev")]
    rev: Option<String>,
}

impl From<Client> for ClientEntity {
    fn from(client: Client) -> Self {
        let id = Guid::from(format!("client:{}", client.client_id()));
        ClientEntity {
            id,
            rev: None,
        }
    }
}

impl Into<Client> for ClientEntity {
    fn into(self) -> Client {
        let guid = &self.id;
        Client::new(guid.id().clone(), None, Scope::default(), vec![])
    }
}