use yarte::Template;

#[derive(Template)]
#[template(path = "index.hbs", err = "index error message")]
pub struct Index<'a> {
    pub name: Option<&'a str>,
}