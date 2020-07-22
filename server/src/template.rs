use askama::Template;

#[derive(Template)]
#[template(path = "redoc.html")]
pub struct ReDoc {
    pub stylesheet_path: String,
    pub favicon_path: String,
    pub spec_url: String,
}
