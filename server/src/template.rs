use askama::Template;

#[derive(Template)]
#[template(path = "swagger/index.html")]
pub struct Swagger {
    pub stylesheet_path: String,
    pub favicon_path: String,
    pub spec_url: String,
}

#[derive(Template)]
#[template(path = "swagger/oauth-redirect.html")]
pub struct SwaggerRedirect;
