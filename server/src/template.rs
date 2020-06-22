use askama::Template;

#[derive(Template)]
#[template(path = "redoc.html")]
pub struct ReDoc {
    pub spec_url: String,
}
