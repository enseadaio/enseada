use std::fmt::{self, Display, Formatter};

use actix::Response;
use actix_web::body::Body;
use actix_web::{HttpResponse, ResponseError};
use http::StatusCode;

use oauth::error::{Error, ErrorKind};
pub use routes::mount;

mod routes;
mod template;

#[derive(Debug)]
pub struct ErrorResponse(oauth::error::Error);

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> Self {
        ErrorResponse(err)
    }
}

impl From<couchdb::error::Error> for ErrorResponse {
    fn from(err: couchdb::error::Error) -> Self {
        let oerr = oauth::error::Error::from(err);
        ErrorResponse(oerr)
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ResponseError for ErrorResponse {
    fn error_response(&self) -> HttpResponse {
        let err = &self.0;
        match err.kind() {
            ErrorKind::AccessDenied => HttpResponse::Forbidden(),
            ErrorKind::InvalidClient => HttpResponse::Unauthorized(),
            ErrorKind::ServerError | ErrorKind::Unknown => HttpResponse::InternalServerError(),
            ErrorKind::TemporarilyUnavailable => HttpResponse::ServiceUnavailable(),
            _ => HttpResponse::BadRequest(),
        }
        .json2(err)
    }
}
