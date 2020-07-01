use actix_web::get;
use actix_web::web::{Data, Query};

use enseada::pagination::Page;

use crate::config::CONFIG;
use crate::couchdb::repository::Repository;
use crate::dashboard::error::DashboardError;
use crate::dashboard::template::OCI;
use crate::http::extractor::user::DashboardUser;
use crate::http::PaginationQuery;
use crate::oci::entity::Repo;
use crate::oci::service::RepoService;

#[get("/dashboard/containers")]
pub async fn index(
    DashboardUser(user): DashboardUser,
    oci: Data<RepoService>,
    page: Query<PaginationQuery>,
) -> Result<OCI, DashboardError> {
    let limit = page.limit();
    let offset = page.offset();

    let repos = oci.list(limit, offset).await?;
    let prev_link = prev_page_link(&repos);
    let next_link = next_page_link(&repos);
    let repos = repos.into_iter().collect();
    Ok(OCI {
        user: Some(user),
        repos,
        oci_url: oci_url(),
        next_link,
        prev_link,
    })
}

fn oci_url() -> String {
    let sub = CONFIG.oci().subdomain();
    let mut host = CONFIG.public_host().clone();
    let base = host.host_str().unwrap();
    let full = format!("{}.{}", sub, base);
    host.set_host(Some(full.as_str())).unwrap();
    host.to_string()
}

fn prev_page_link(page: &Page<Repo>) -> Option<String> {
    if page.is_first() {
        None
    } else {
        Some(format!(
            "/dashboard/containers?limit={}&offset={}",
            page.limit(),
            page.offset() - 1
        ))
    }
}

fn next_page_link(page: &Page<Repo>) -> Option<String> {
    if page.is_last() {
        None
    } else {
        Some(format!(
            "/dashboard/containers?limit={}&offset={}",
            page.limit(),
            page.offset() + 1
        ))
    }
}
