use actix_web::web::ServiceConfig;
use actix_web::{guard, web, HttpResponse, Resource};

use crate::assets::handle_embedded_file;
use crate::{assets, routes};

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
