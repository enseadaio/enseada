use actix_web::web;

use crate::docker::handler::test;
use crate::http::guard::subdomain;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v2")
        .guard(subdomain("docker"))
        .route("/test", web::get().to(test))
    );
}


