use askama::Template;

use crate::user::User;

#[derive(Template)]
#[template(path = "dashboard/index.html")]
pub struct Index {
    pub user: Option<User>,
    pub oci_repos_count: usize,
    pub maven_repos_count: usize,
}

#[derive(Template)]
#[template(path = "dashboard/error.html")]
pub struct ErrorPage {
    pub user: Option<User>,
    pub reason: String,
    pub message: String,
}

impl ErrorPage {
    pub fn new(reason: String, message: String) -> Self {
        ErrorPage {
            user: None,
            reason,
            message,
        }
    }
}
