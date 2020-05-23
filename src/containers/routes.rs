use actix_web::{ResponseError, web};
use actix_web::middleware::DefaultHeaders;

use crate::config::CONFIG;
use crate::containers::{handler, header};
use crate::containers::error::{Error, ErrorCode};
use crate::containers::manifest::resolver::add_manifest_resolver;
use crate::containers::upload::add_upload_service;
use crate::http::guard::subdomain;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let domain = CONFIG.oci().subdomain();

    cfg.service(web::scope("/v2")
        .guard(subdomain(domain))
        .configure(add_manifest_resolver)
        .configure(add_upload_service)
        .wrap(DefaultHeaders::new()
            .header(header::DISTRIBUTION_API_VERSION, "registry/2.0")
        )
        .route("/", web::get().to(handler::check_version))
        .service(web::scope("/{group}/{name}")
            .service(web::resource("/manifests/{ref}")
                .route(web::head().to(handler::manifest::get))
                .route(web::get().to(handler::manifest::get))
            )
            .service(web::scope("/blobs")
                .route("/uploads/", web::post().to(handler::upload::start))
                .service(web::resource("/uploads/{upload_id}")
                    .name("upload")
                    .route(web::head().to(handler::upload::get_status))
                    .route(web::get().to(handler::upload::get_status))
                    .route(web::patch().to(handler::upload::upload_chunk))
                    .route(web::put().to(handler::upload::complete))
                    .route(web::delete().to(handler::upload::cancel))
                )
                .service(web::resource("/{digest}")
                    .name("blob")
                    .route(web::head().to(handler::upload::exists))
                )
            )
            .route("/test", web::get().to(handler::test))
        )
        .default_service(web::route().to(|| Error::from(ErrorCode::NotFound).error_response())));
}

