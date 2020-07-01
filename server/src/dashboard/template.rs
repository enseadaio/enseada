use askama::Template;

use crate::oci::entity::Repo;
use crate::user::User;

#[derive(Template)]
#[template(path = "dashboard/index.html")]
pub struct Index {
    pub user: Option<User>,
    pub oci_repos_count: usize,
    pub maven_repos_count: usize,
}

#[derive(Template)]
#[template(path = "dashboard/oci/index.html")]
pub struct OCI {
    pub user: Option<User>,
    pub repos: Vec<Repo>,
    pub oci_url: String,
    pub next_link: Option<String>,
    pub prev_link: Option<String>,
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
