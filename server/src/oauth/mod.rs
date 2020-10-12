use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use actix::Response;
use actix_web::body::Body;
use actix_web::{HttpResponse, ResponseError};
use http::StatusCode;
use url::ParseError;

use actix_web::error::UrlGenerationError;
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

impl From<actix_web::error::Error> for ErrorResponse {
    fn from(err: actix_web::error::Error) -> Self {
        let err = Error::new(ErrorKind::ServerError, err);
        ErrorResponse(err)
    }
}

impl From<url::ParseError> for ErrorResponse {
    fn from(err: ParseError) -> Self {
        let err = Error::new(ErrorKind::InvalidRequest, err);
        ErrorResponse(err)
    }
}

impl From<UrlGenerationError> for ErrorResponse {
    fn from(err: UrlGenerationError) -> Self {
        let err = Error::new(ErrorKind::ServerError, err);
        ErrorResponse(err)
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
            ErrorKind::AuthenticationFailed | ErrorKind::InvalidClient => {
                HttpResponse::Unauthorized()
            }
            ErrorKind::ServerError | ErrorKind::Unknown => HttpResponse::InternalServerError(),
            ErrorKind::TemporarilyUnavailable => HttpResponse::ServiceUnavailable(),
            _ => HttpResponse::BadRequest(),
        }
        .json2(err)
    }
}

impl Deref for ErrorResponse {
    type Target = oauth::error::Error;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
