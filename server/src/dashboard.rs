use actix_files as fs;
use actix_files::NamedFile;
use actix_web::web::ServiceConfig;
use actix_web::{guard, web, HttpResponse, Resource, Result};

use crate::routes;

static DIST_PATH: &str = "../dashboard/dist";

pub fn mount(cfg: &mut ServiceConfig) {
    cfg.service(fs::Files::new("/static", DIST_PATH));
}

async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open(format!("{}/index.html", DIST_PATH))?)
}

pub fn default_service() -> Resource {
    let not_found_guard = guard::Any(guard::fn_guard(|req| {
        let path = req.uri.path();
        log::warn!("Path {}", path);
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
