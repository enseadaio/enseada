use actix_files as fs;
use actix_web::{FromRequest, web};

use crate::docker::handler::test;
use crate::http::guard::Subdomain;
use crate::http::handler::{api_docs, health, oauth, ui, user};
use crate::oauth::request::{AuthorizationRequest, TokenRequest};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v2")
        .guard(Subdomain("docker"))
        .route("/test", web::get().to(test)))
    ;
}


