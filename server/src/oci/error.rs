use std::fmt::{self, Display, Formatter};

use actix_web::{HttpResponse, ResponseError};
use http::{header, StatusCode};
use serde::Serialize;

use oci::error::{Error, ErrorCode};
use oci::mime::MediaType;

#[derive(Debug, Serialize)]
pub struct ErrorResponse(Error);

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> Self {
        ErrorResponse(err)
    }
}

#[derive(Default, Serialize)]
pub struct ErrorBody {
    errors: Vec<Error>,
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        self.0.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let body = ErrorBody {
            errors: vec![self.0.clone()],
        };

        let mut res = HttpResponse::build(self.status_code());
        let mut accept = Vec::new();
        accept.extend(MediaType::ImageIndex.compatible_types());
        accept.extend(MediaType::ImageManifest.compatible_types());
        res.header(header::ACCEPT, accept.join(","));
        if let ErrorCode::RequestedRangeNotSatisfiable(range) = self.0.code() {
            res.header(header::RANGE, format!("0-{}", range));
        }
        res.json(body)
    }
}

impl From<enseada::couchdb::error::Error> for ErrorResponse {
    fn from(err: enseada::couchdb::error::Error) -> Self {
        ErrorResponse(Error::from(err))
    }
}

impl From<hold::error::Error> for ErrorResponse {
    fn from(err: hold::error::Error) -> Self {
        ErrorResponse(Error::from(err))
    }
}
