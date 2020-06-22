use actix_web::web::{Data, Query, ServiceConfig};
use actix_web::{get, HttpResponse};
use serde::Deserialize;

use crate::couchdb::repository::Repository;
use crate::dashboard::error::DashboardError;
use crate::dashboard::template::{ErrorPage, Index};
use crate::http::extractor::user::DashboardUser;
use crate::oci::service::RepoService;

pub fn mount(cfg: &mut ServiceConfig) {
    cfg.service(index);
    cfg.service(auth_callback);
}

#[derive(Debug, Deserialize)]
pub struct OAuthError {
    error: String,
    error_description: String,
}

#[get("/dashboard")]
pub async fn index(
    DashboardUser(user): DashboardUser,
    oci_repos: Data<RepoService>,
) -> Result<Index, DashboardError> {
    let oci_repos_count = oci_repos.count().await?;
    Ok(Index {
        user: Some(user),
        oci_repos_count,
        maven_repos_count: 0,
    })
}

#[get("/dashboard/auth/callback")]
pub async fn auth_callback(query: Option<Query<OAuthError>>) -> HttpResponse {
    match query {
        Some(err) => HttpResponse::Unauthorized().body(
            ErrorPage::new("Unauthorized".to_string(), err.error_description.clone()).to_string(),
        ),
        None => HttpResponse::SeeOther()
            .header(http::header::LOCATION, "/dashboard")
            .finish(),
    }
}
