use crate::oauth::{Result, Scope};

pub trait Client {
    fn client_id() -> String;
    fn client_secret() -> Option<String>;
    fn allowed_scopes() -> Scope;
    fn allowed_redirect_uris() -> Vec<url::Url>;
}
