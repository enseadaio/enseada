use actix_web::web::Json;
use crate::errors::ApiError;

pub fn ok<T>(data: T) -> Result<Json<T>, ApiError> {
    Ok(Json(data))
}