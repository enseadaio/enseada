use askama::Template;

pub mod oauth;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub name: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "redoc.html")]
pub struct ReDoc {
    pub spec_url: String,
}