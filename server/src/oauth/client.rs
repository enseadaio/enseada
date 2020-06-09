use std::collections::HashSet;

use enseada::secure;

use crate::oauth::client::ClientKind::{Confidential, Public};
use crate::oauth::error::{Error, ErrorKind};
use crate::oauth::scope::Scope;
use crate::oauth::Result;

#[derive(Clone, Debug)]
pub enum ClientKind {
    Public,
    Confidential { secret: String },
}

#[derive(Debug, Clone)]
pub struct Client {
    client_id: String,
    kind: ClientKind,
    allowed_scopes: Scope,
    allowed_redirect_uris: HashSet<url::Url>,
}

impl Client {
    pub fn confidential(
        client_id: String,
        client_secret: String,
        allowed_scopes: Scope,
        allowed_redirect_uris: HashSet<url::Url>,
    ) -> Result<Client> {
        let secret = secure::hash_password(client_secret.as_str())
            .map_err(|msg| Error::new(ErrorKind::InvalidClient, msg))?;
        Ok(Self::confidential_with_hash(
            client_id,
            secret,
            allowed_scopes,
            allowed_redirect_uris,
        ))
    }

    pub fn confidential_with_hash(
        client_id: String,
        client_secret_hash: String,
        allowed_scopes: Scope,
        allowed_redirect_uris: HashSet<url::Url>,
    ) -> Client {
        Client {
            client_id,
            kind: Confidential {
                secret: client_secret_hash,
            },
            allowed_scopes,
            allowed_redirect_uris,
        }
    }

    pub fn public(
        client_id: String,
        allowed_scopes: Scope,
        allowed_redirect_uris: HashSet<url::Url>,
    ) -> Client {
        Client {
            client_id,
            kind: Public,
            allowed_scopes,
            allowed_redirect_uris,
        }
    }

    pub fn client_id(&self) -> &String {
        &self.client_id
    }

    pub fn kind(&self) -> &ClientKind {
        &self.kind
    }

    pub fn allowed_scopes(&self) -> &Scope {
        &self.allowed_scopes
    }

    pub fn allowed_redirect_uris(&self) -> &HashSet<url::Url> {
        &self.allowed_redirect_uris
    }
}
