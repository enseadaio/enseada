use actix_web::http::header;
use actix_web::HttpResponse;
use actix_web::web::Json;

use crate::errors::ApiError;

pub fn ok<T>(data: T) -> Result<Json<T>, ApiError> {
    Ok(Json(data))
}

pub fn redirect_to(location: impl ToString) -> HttpResponse {
    HttpResponse::Found()
        .header(header::LOCATION, location.to_string())
        .finish()
        .into_body()
}
