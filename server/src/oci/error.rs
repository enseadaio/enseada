use std::fmt::{self, Display, Formatter};

use actix_web::body::{Body, MessageBody};
use actix_web::dev::ServiceResponse;
use actix_web::middleware::errhandlers::ErrorHandlerResponse;
use actix_web::{HttpResponse, ResponseError};
use http::{header, HeaderValue, StatusCode};
use serde::Serialize;

use oauth::scope::Scope;
use oci::error::{Error, ErrorCode};
use oci::mime::MediaType;
use rbac::EvaluationError;

use crate::config::CONFIG;
use enseada::storage;

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

impl From<storage::error::Error> for ErrorResponse {
    fn from(err: storage::error::Error) -> Self {
        ErrorResponse(Error::from(err))
    }
}

impl From<oauth::error::Error> for ErrorResponse {
    fn from(err: oauth::error::Error) -> Self {
        ErrorResponse(Error::new(ErrorCode::Denied, err))
    }
}

impl From<EvaluationError> for ErrorResponse {
    fn from(err: EvaluationError) -> Self {
        ErrorResponse(Error::new(ErrorCode::Denied, err))
    }
}

#[derive(Debug)]
struct UnauthorizedError {
    realm: String,
    service: String,
    scope: Scope,
}

impl Default for UnauthorizedError {
    fn default() -> Self {
        UnauthorizedError {
            realm: "Enseada OAuth Server".to_string(),
            service: "Enseada OCI Registry".to_string(),
            scope: Scope::from(vec!["oci:image:push:", "oci:image:pull"]),
        }
    }
}

impl Display for UnauthorizedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        "access to the requested resource is not authorized".fmt(f)
    }
}

impl ResponseError for UnauthorizedError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn error_response(&self) -> HttpResponse {
        let challenge = format!(
            "Basic realm=\"{}\",service=\"{}\",scope=\"{}\"",
            &self.realm, &self.service, &self.scope,
        );
        HttpResponse::Unauthorized()
            .header(http::header::WWW_AUTHENTICATE, challenge)
            .json(ErrorBody {
                errors: vec![Error::new(
                    ErrorCode::Unauthorized,
                    "access to the requested resource is not authorized",
                )],
            })
    }
}

pub fn handle_unauthorized_request<B: MessageBody>(
    res: ServiceResponse<B>,
) -> actix_web::error::Result<ErrorHandlerResponse<B>> {
    let err = UnauthorizedError::default();
    Ok(ErrorHandlerResponse::Response(res.error_response(err)))
}
