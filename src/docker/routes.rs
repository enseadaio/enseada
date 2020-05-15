use actix_web::web;
use actix_web::middleware::DefaultHeaders;

use crate::docker::{handler, header};
use crate::docker::manifest::resolver::add_manifest_resolver;
use crate::http::guard::subdomain;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v2")
        .guard(subdomain("docker"))
        .configure(add_manifest_resolver)
        .wrap(DefaultHeaders::new()
            .header(header::DISTRIBUTION_API_VERSION, "registry/2.0")
        )
        .route("/", web::get().to(handler::check_version))
        .service(web::scope("/{group}/{name}")
            .service(web::resource("/manifests/{ref}")
                .route(web::get().to(handler::manifest::get))
            )
            .route("/test", web::get().to(handler::test))
        )
    );
}


