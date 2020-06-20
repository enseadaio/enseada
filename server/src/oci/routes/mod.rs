use std::sync::Arc;

use actix_web::get;
use actix_web::middleware::DefaultHeaders;
use actix_web::web::{self, ServiceConfig};
use actix_web::HttpResponse;
use serde::Deserialize;

use crate::config::CONFIG;
use crate::http::guard::subdomain;
use crate::oci::header;
use crate::oci::service::{RepoService, UploadService};
use crate::storage;

mod api;
mod upload;

pub fn mount(cfg: &mut ServiceConfig) {
    let couch = &crate::couchdb::SINGLETON;
    let db = Arc::new(couch.database(crate::couchdb::name::OCI, true));
    let provider = Arc::new(storage::new_provider().expect("docker storage provider"));

    let repo = RepoService::new(db.clone());
    cfg.data(repo);

    let repo = UploadService::new(db.clone(), provider.clone());
    cfg.data(repo);

    cfg.service(api::list_repos);
    cfg.service(api::create_repo);
    cfg.service(api::get_repo);
    cfg.service(api::delete_repo);

    let sub = CONFIG.oci().subdomain();
    cfg.service(
        web::scope("/v2")
            .guard(subdomain(sub))
            .wrap(DefaultHeaders::new().header(header::DISTRIBUTION_API_VERSION, "registry/2.0"))
            .service(root)
            .service(upload::start)
            .service(upload::get)
            .service(upload::push)
            .service(upload::complete)
            .service(upload::delete),
    );
}

#[derive(Debug, Deserialize)]
pub struct RepoPath {
    group: String,
    name: String,
}

#[get("/")]
pub async fn root() -> HttpResponse {
    HttpResponse::Ok().finish()
}
