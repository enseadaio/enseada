use std::borrow::Cow;

use actix_web::body::Body;
use actix_web::HttpResponse;
use mime_guess::from_path;
use rust_embed::RustEmbed;

pub const PATH_PREFIX: &str = "/static";

#[derive(RustEmbed)]
#[folder = "../dashboard/dist"]
pub struct Asset;

pub fn handle_embedded_file(path: &str) -> HttpResponse {
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

pub fn stylesheet_path() -> String {
    find_asset(|f| f.ends_with(".css")).expect("stylesheet path not found")
}

pub fn icon_path() -> String {
    find_asset(|f| f.starts_with("favicon")).expect("icon path not found")
}

pub fn logo_path() -> String {
    find_asset(|f| f.starts_with("enseada-logo")).expect("logo path not found")
}

fn find_asset<F: Fn(&str) -> bool>(fun: F) -> Option<String> {
    for file in Asset::iter() {
        if fun(&file) {
            return Some(format!("{}/{}", PATH_PREFIX, file));
        }
    }

    None
}
