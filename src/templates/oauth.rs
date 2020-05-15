use askama::Template;

#[derive(Template)]
#[template(path = "oauth/login.html")]
pub struct LoginForm {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: String,
}