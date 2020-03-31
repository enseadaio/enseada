use crate::oauth::{Result, Scope};

pub trait Client {
    fn client_id(&self) -> String;
    fn client_secret(&self) -> Option<String>;
    fn allowed_scopes(&self) -> Scope;
    fn allowed_redirect_uris(&self) -> Vec<url::Url>;
}
