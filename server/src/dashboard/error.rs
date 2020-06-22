use actix_web::{HttpResponse, ResponseError};
use http::StatusCode;
use snafu::Snafu;
use uuid::Uuid;

use enseada::error::Error;

use crate::config::CONFIG;
use crate::dashboard::template::ErrorPage;

#[derive(Debug, Snafu)]
pub enum DashboardError {
    #[snafu(display("unauthorized"))]
    Unauthorized,
    #[snafu(display("{}", msg))]
    NotFound { msg: String },
    #[snafu(display("{}", msg))]
    InternalServerError { msg: String },
}

impl DashboardError {
    pub fn new(status: StatusCode, msg: String) -> Self {
        match status {
            StatusCode::UNAUTHORIZED => DashboardError::Unauthorized,
            StatusCode::NOT_FOUND => DashboardError::NotFound { msg },
            _ => DashboardError::InternalServerError { msg },
        }
    }
}

impl ResponseError for DashboardError {
    fn status_code(&self) -> StatusCode {
        match self {
            DashboardError::Unauthorized => StatusCode::UNAUTHORIZED,
            DashboardError::NotFound { .. } => StatusCode::NOT_FOUND,
            DashboardError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            DashboardError::Unauthorized => HttpResponse::SeeOther()
                .header(http::header::LOCATION, build_oauth_url())
                .finish(),
            _ => HttpResponse::build(self.status_code()).body(
                ErrorPage::new(
                    self.status_code()
                        .canonical_reason()
                        .unwrap_or("unknown error")
                        .to_string(),
                    self.to_string(),
                )
                .to_string(),
            ),
        }
    }
}

impl From<Error> for DashboardError {
    fn from(err: Error) -> Self {
        let msg = err.to_string();
        match err.status() {
            StatusCode::UNAUTHORIZED => DashboardError::Unauthorized,
            _ => DashboardError::InternalServerError { msg },
        }
    }
}

impl From<couchdb::error::Error> for DashboardError {
    fn from(err: couchdb::error::Error) -> Self {
        let msg = err.to_string();
        match err.status() {
            StatusCode::NOT_FOUND => DashboardError::NotFound { msg },
            _ => DashboardError::InternalServerError { msg },
        }
    }
}

impl From<actix_web::Error> for DashboardError {
    fn from(err: actix_web::Error) -> Self {
        Self::from(&err)
    }
}

impl From<&actix_web::Error> for DashboardError {
    fn from(err: &actix_web::Error) -> Self {
        let err = err.as_response_error();
        Self::new(err.status_code(), err.to_string())
    }
}

fn build_oauth_url() -> String {
    let host = CONFIG.public_host();
    let mut url = host.join("/oauth/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", "enseada")
        .append_pair("scope", "profile")
        .append_pair(
            "redirect_uri",
            &host.join("/dashboard/auth/callback").unwrap().to_string(),
        )
        .append_pair("state", &Uuid::new_v4().to_string())
        .append_pair("response_type", "code");
    url.to_string()
}
