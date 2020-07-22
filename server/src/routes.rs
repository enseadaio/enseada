use actix_files as fs;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder, ResponseError};

use crate::assets;
use crate::http::error::ApiError;
use crate::http::header::accept;
use crate::template::ReDoc;

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.service(open_api);
    cfg.service(redoc);
}

const SPEC: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi.yml"));

#[get("/api/docs/openapi.yml")]
pub async fn open_api() -> HttpResponse {
    HttpResponse::Ok().content_type("text/yaml").body(SPEC)
}

#[get("/api/docs")]
pub async fn redoc() -> impl Responder {
    ReDoc {
        stylesheet_path: assets::stylesheet_path(),
        favicon_path: assets::icon_path(),
        spec_url: "/api/docs/openapi.yml".to_string(),
    }
}

pub async fn not_found(req: HttpRequest) -> HttpResponse {
    let msg = format!("Route {} not found", req.path());
    ApiError::NotFound(msg).error_response()
}
