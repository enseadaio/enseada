use std::borrow::Cow;

use actix_files as fs;
use actix_web::body::Body;
use actix_web::web::ServiceConfig;
use actix_web::{guard, web, HttpResponse, Resource, Result};
use mime_guess::from_path;
use rust_embed::RustEmbed;

use crate::{assets, routes};

#[derive(RustEmbed)]
#[folder = "../dashboard/dist"]
pub struct Asset;

pub fn mount(cfg: &mut ServiceConfig) {
    cfg.service(
        web::resource(format!("{}/{{_:.*}}", assets::PATH_PREFIX))
            .route(web::get().to(static_files)),
    );
}

async fn index() -> HttpResponse {
    handle_embedded_file("index.html")
}

fn static_files(path: web::Path<String>) -> HttpResponse {
    handle_embedded_file(&path.into_inner())
}

fn handle_embedded_file(path: &str) -> HttpResponse {
    log::info!("Handling embedded file {}", path);
    match Asset::get(path) {
        Some(content) => {
            let body: Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };
            HttpResponse::Ok()
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

pub fn default_service() -> Resource {
    let not_found_guard = guard::Any(guard::fn_guard(|req| {
        let path = req.uri.path();
        path.starts_with("/api")
    }))
    .or(guard::fn_guard(|req| req.uri.path().starts_with("/oauth")));
    web::resource("")
        .route(web::route().guard(not_found_guard).to(routes::not_found))
        .route(web::get().to(index))
        .route(
            web::route()
                .guard(guard::Not(guard::Get()))
                .to(HttpResponse::MethodNotAllowed),
        )
}
