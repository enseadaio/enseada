use crate::oauth::client::Client;
use crate::oauth::Scope;
use url::Url;

pub struct OAuthClient {
    // TODO: implement
}

impl Client for OAuthClient {
    fn client_id(&self) -> String {
        "client".to_string()
    }

    fn client_secret(&self) -> Option<String> {
        Some("secret".to_string())
    }

    fn allowed_scopes(&self) -> Scope {
        Scope::from("profile")
    }

    fn allowed_redirect_uris(&self) -> Vec<Url> {
        vec!(Url::parse("http://example.com").unwrap())
    }
}