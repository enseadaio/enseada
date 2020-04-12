use yarte::Template;

pub mod oauth;

#[derive(Template)]
#[template(path = "index.hbs", err = "index error message")]
pub struct Index<'a> {
    pub name: Option<&'a str>,
}
