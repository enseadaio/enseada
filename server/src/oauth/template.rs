use askama::Template;

use oauth::request::PkceRequest;

use crate::assets;

#[derive(Template)]
#[template(path = "oauth/login.html")]
pub struct LoginForm {
    pub stylesheet_path: String,
    pub favicon_path: String,
    pub logo_path: String,
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: String,
    pub error: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Template)]
#[template(path = "oauth/logout.html")]
pub struct Logout {
    pub stylesheet_path: String,
    pub favicon_path: String,
}

impl Default for Logout {
    fn default() -> Self {
        Logout {
            stylesheet_path: assets::stylesheet_path(),
            favicon_path: assets::icon_path(),
        }
    }
}
