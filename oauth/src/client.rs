use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

use crate::scope::Scope;

#[derive(Clone, Debug)]
pub enum ClientKind {
    Public,
    Confidential,
}

impl Display for ClientKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            ClientKind::Public => "public",
            ClientKind::Confidential => "confidential",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    client_id: String,
    kind: ClientKind,
    allowed_scopes: Scope,
    allowed_redirect_uris: HashSet<url::Url>,
}

impl Client {
    pub fn new(
        client_id: String,
        kind: ClientKind,
        allowed_scopes: Scope,
        allowed_redirect_uris: HashSet<url::Url>,
    ) -> Client {
        Client {
            client_id,
            kind,
            allowed_scopes,
            allowed_redirect_uris,
        }
    }

    pub fn client_id(&self) -> &str {
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

    pub fn set_allowed_scopes(&mut self, scopes: Scope) -> &mut Self {
        self.allowed_scopes = scopes;
        self
    }

    pub fn set_allowed_redirect_uris(&mut self, uris: HashSet<url::Url>) -> &mut Self {
        self.allowed_redirect_uris = uris;
        self
    }
}
