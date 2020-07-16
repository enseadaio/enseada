use actix_web::http::header;
use actix_web::web::Json;
use actix_web::HttpResponse;

use enseada::guid::Guid;

use crate::http::error::ApiError;

pub fn ok<T>(data: T) -> Result<Json<T>, ApiError> {
    Ok(Json(data))
}

pub fn redirect_to<S: ToString>(location: S) -> HttpResponse {
    HttpResponse::Found()
        .header(header::LOCATION, location.to_string())
        .finish()
        .into_body()
}

pub fn not_found<T>(id: &Guid) -> Result<Json<T>, ApiError> {
    let kind = id.partition().unwrap_or_else(|| "resource");
    Err(ApiError::NotFound(format!(
        "{} '{}' not found",
        kind,
        id.id()
    )))
}

pub fn conflict<T>(msg: String) -> Result<Json<T>, ApiError> {
    Err(ApiError::Conflict(msg))
}
