use axum::response::{IntoResponse, Response};

use axum::Json;
use cqrs::command::ValidationErrors;
use http::StatusCode;
use serde::Serialize;

use futon::FutonError;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
    inner: Option<anyhow::Error>,
}

impl ApiError {
    fn new(status: StatusCode, err: anyhow::Error) -> Self {
        Self {
            status,
            message: err.to_string(),
            inner: Some(err),
        }
    }

    pub fn not_found(msg: impl ToString) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.to_string(),
            inner: None,
        }
    }
}

pub trait ToStatusCode {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl ToStatusCode for FutonError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<E> From<E> for ApiError
where
    E: ToStatusCode + Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let status = err.status_code();
        Self::new(status, err.into())
    }
}

impl ToStatusCode for anyhow::Error {
    fn status_code(&self) -> StatusCode {
        for cause in self.chain() {
            if let Some(err) = cause.downcast_ref::<FutonError>() {
                return err.status_code();
            }
        }

        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: errors
                .errors()
                .iter()
                .map(|err| err.message().to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            inner: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasons: Option<Vec<String>>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let reasons = self.inner.and_then(|err| {
            if err.chain().len() > 1 {
                Some(err.chain().skip(1).map(|cause| cause.to_string()).collect())
            } else {
                None
            }
        });

        (
            self.status,
            Json(ApiErrorResponse {
                message: self.message,
                reasons,
            }),
        )
            .into_response()
    }
}
