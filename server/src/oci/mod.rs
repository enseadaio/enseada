use std::sync::{Arc, RwLock};

use actix_web::middleware::errhandlers::ErrorHandlers;
use actix_web::middleware::DefaultHeaders;
use actix_web::web::{self, ServiceConfig};
use actix_web::{get, guard, FromRequest, ResponseError};
use actix_web::{HttpRequest, HttpResponse};
use http::StatusCode;
use serde::Deserialize;

use enseada::couchdb::db::Database;
use enseada::storage::Provider;
use events::EventBus;
use oci::header;
use oci::service::{BlobService, ManifestService, RepoService, UploadService};

use crate::config::{Configuration, CONFIG};
use crate::http::extractor::session::TokenSession;
use crate::storage;

mod api;
mod blob;
mod error;
mod manifest;
mod tag;
mod upload;

pub type Result<T> = std::result::Result<T, error::ErrorResponse>;

pub fn mount(
    cfg: &Configuration,
    db: Database,
    bus: Arc<RwLock<EventBus>>,
    store: Arc<Provider>,
) -> Box<impl FnOnce(&mut ServiceConfig)> {
    let host = cfg.oci().host();
    let max_body_size = cfg.oci().max_body_size();

    Box::new(move |cfg: &mut ServiceConfig| {
        let db = Arc::new(db);

        let repo = RepoService::new(db.clone(), bus.clone());
        cfg.data(repo);

        let mut bus = bus.write().expect("oci::mount EventBus unlock");

        let repo = UploadService::new(db.clone(), store.clone());
        cfg.data(repo);
        let repo_handler = UploadService::new(db.clone(), store.clone());
        bus.subscribe(repo_handler);

        let blob = BlobService::new(db.clone(), store.clone());
        cfg.data(blob);
        let blob_handler = BlobService::new(db.clone(), store.clone());
        bus.subscribe(blob_handler);

        let manifest = ManifestService::new(db.clone());
        cfg.data(manifest);
        let manifest_handler = ManifestService::new(db.clone());
        bus.subscribe(manifest_handler);

        cfg.service(api::list_repos);
        cfg.service(api::create_repo);
        cfg.service(api::get_repo);
        cfg.service(api::delete_repo);

        cfg.service(
            web::scope("/v2")
                .guard(guard::Host(host))
                .wrap(
                    DefaultHeaders::new().header(header::DISTRIBUTION_API_VERSION, "registry/2.0"),
                )
                .wrap(
                    ErrorHandlers::new()
                        .handler(StatusCode::UNAUTHORIZED, error::handle_unauthorized_request),
                )
                .app_data(actix_web::web::Bytes::configure(|cfg| {
                    cfg.limit(max_body_size)
                }))
                .service(root)
                // Tags
                .service(tag::list)
                // Manifests
                .service(
                    web::resource("/{group}/{name}/manifests/{reference}")
                        .route(web::get().to(manifest::get))
                        .route(web::head().to(manifest::get))
                        .route(web::put().to(manifest::put))
                        .route(web::delete().to(manifest::delete)),
                )
                // Blobs
                .service(blob::get)
                .service(blob::head)
                .service(blob::delete)
                // Uploads
                .service(upload::start)
                .service(upload::get)
                .service(upload::push)
                .service(upload::complete)
                .service(upload::delete),
        );
    })
}

#[derive(Debug, Deserialize)]
pub struct RepoPath {
    group: String,
    name: String,
}

#[get("")]
pub async fn root(session: TokenSession) -> HttpResponse {
    log::debug!("{:?}", session);
    HttpResponse::Ok().finish()
}
