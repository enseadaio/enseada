use std::collections::HashSet;
use std::convert::TryInto;

use serde::{Deserialize, Serialize};
use url::Url;

use enseada::guid::Guid;

use crate::oauth::client::Client;
use crate::oauth::client::ClientKind as ExtClientKind;
use crate::oauth::error::Error;
use crate::oauth::scope::Scope;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Public,
    Confidential,
}

impl From<ExtClientKind> for ClientKind {
    fn from(kind: ExtClientKind) -> Self {
        match kind {
            ExtClientKind::Public => ClientKind::Public,
            ExtClientKind::Confidential { .. } => ClientKind::Confidential,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientEntity {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    kind: ClientKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret_hash: Option<String>,
    allowed_scopes: Scope,
    allowed_redirect_uris: HashSet<Url>,
}

impl ClientEntity {
    pub fn build_guid(client_id: &str) -> Guid {
        Guid::from(format!("client:{}", client_id))
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn rev(&self) -> Option<String> {
        self.rev.clone()
    }
}

impl From<Client> for ClientEntity {
    fn from(client: Client) -> Self {
        let id = Self::build_guid(client.client_id());
        let kind = client.kind().clone();
        ClientEntity {
            id,
            rev: None,
            client_secret_hash: match &kind {
                ExtClientKind::Public => None,
                ExtClientKind::Confidential { secret } => Some(secret.clone()),
            },
            kind: ClientKind::from(kind),
            allowed_scopes: client.allowed_scopes().clone(),
            allowed_redirect_uris: client.allowed_redirect_uris().clone(),
        }
    }
}

impl TryInto<Client> for ClientEntity {
    type Error = Error;

    fn try_into(self) -> Result<Client, Self::Error> {
        let guid = &self.id;
        let allowed_redirect_uris = self.allowed_redirect_uris.clone();
        let client_id = guid.id().to_string();
        let scopes = self.allowed_scopes.clone();
        let client = match &self.kind {
            ClientKind::Public => Client::public(client_id, scopes, allowed_redirect_uris),
            ClientKind::Confidential => {
                let secret = self.client_secret_hash.unwrap();
                Client::confidential_with_hash(client_id, secret, scopes, allowed_redirect_uris)
            }
        };
        Ok(client)
    }
}
