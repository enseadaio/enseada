use actix_web::{get, web, HttpRequest, HttpResponse, Responder, ResponseError};

use crate::assets;
use crate::http::error::ApiError;
use crate::template::{Swagger, SwaggerRedirect};

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.service(open_api);
    cfg.service(api_docs);
    cfg.service(api_docs_redirect);
}

#[get("/api/docs/openapi.yml")]
pub async fn open_api() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/yaml")
        .body(api::SPEC_YML)
}

#[get("/api/docs")]
pub async fn api_docs() -> impl Responder {
    Swagger {
        stylesheet_path: assets::stylesheet_path(),
        favicon_path: assets::icon_path(),
        spec_url: "/api/docs/openapi.yml".to_string(),
    }
}

#[get("/api/docs/oauth-redirect")]
pub async fn api_docs_redirect() -> impl Responder {
    SwaggerRedirect
}

pub async fn not_found(req: HttpRequest) -> HttpResponse {
    let msg = format!("Route {} not found", req.path());
    ApiError::NotFound(msg).error_response()
}
