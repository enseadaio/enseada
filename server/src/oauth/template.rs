use crate::assets;
use askama::Template;

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
