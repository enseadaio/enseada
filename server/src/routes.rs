use actix_files as fs;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::templates::ReDoc;

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.service(home);
    cfg.service(fs::Files::new("/static", "./dist"));
    cfg.service(fs::Files::new("/images", "./images"));
    cfg.service(open_api);
    cfg.service(redoc);
}

#[get("/")]
pub async fn home(req: HttpRequest) -> HttpResponse {
    let accept = req
        .headers()
        .get(http::header::ACCEPT)
        .and_then(|accept| accept.to_str().ok())
        .map(str::to_lowercase)
        .filter(|accept| (*accept).contains("html"));
    let redirect = match accept {
        Some(_) => "/ui",
        None => "/health",
    };
    HttpResponse::SeeOther()
        .header(http::header::LOCATION, redirect)
        .finish()
}

const SPEC: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi.yml"));

#[get("/api/docs/openapi.yml")]
pub async fn open_api() -> HttpResponse {
    HttpResponse::Ok().content_type("text/yaml").body(SPEC)
}

#[get("/api/docs")]
pub async fn redoc() -> impl Responder {
    ReDoc {
        spec_url: "/api/docs/openapi.yml".to_string(),
    }
}
