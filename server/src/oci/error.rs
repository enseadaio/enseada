use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use actix_web::dev::ServiceResponse;
use actix_web::middleware::errhandlers::ErrorHandlerResponse;
use actix_web::{HttpResponse, ResponseError};
use http::{header, StatusCode};
use serde::{Deserialize, Serialize};

use couchdb::error::Error as CouchError;

use crate::http::error::ApiError;
use crate::oci::mime::MediaType;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Error {
    code: ErrorCode,
    message: String,
    detail: HashMap<String, String>,
}

impl Error {
    pub fn new(code: ErrorCode, message: &str) -> Self {
        Error {
            code,
            message: message.to_string(),
            detail: HashMap::new(),
        }
    }
}

impl From<ErrorCode> for Error {
    fn from(code: ErrorCode) -> Self {
        match code {
            ErrorCode::BlobUnknown => Self::new(code, "blob unknown to registry"),
            ErrorCode::BlobUploadInvalid => Self::new(code, "blob upload invalid"),
            ErrorCode::BlobUploadUnknown => Self::new(code, "blob upload unknown to registry"),
            ErrorCode::DigestInvalid => {
                Self::new(code, "provided digest did not match uploaded content")
            }
            ErrorCode::ManifestBlobUnknown => Self::new(code, "blob unknown to registry"),
            ErrorCode::ManifestInvalid => Self::new(code, "manifest invalid"),
            ErrorCode::ManifestUnknown => Self::new(code, "manifest unknown"),
            ErrorCode::ManifestUnverified => {
                Self::new(code, "manifest failed signature verification")
            }
            ErrorCode::NameInvalid => Self::new(code, "invalid repository name"),
            ErrorCode::NameUnknown => Self::new(code, "repository name not known to registry"),
            ErrorCode::SizeInvalid => {
                Self::new(code, "provided length did not match content length")
            }
            ErrorCode::TagInvalid => Self::new(code, "manifest tag did not match URI"),
            ErrorCode::Unauthorized => Self::new(code, "authentication required"),
            ErrorCode::Denied => Self::new(code, "requested access to the resource is denied"),
            ErrorCode::Unsupported => Self::new(code, "The operation is unsupported"),
            ErrorCode::MediaTypeUnsupported => Self::new(code, "The media type is unsupported"),
            ErrorCode::NotFound => Self::new(code, "Not found"),
            ErrorCode::Internal => Self::new(code, "Internal server error"),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<CouchError> for Error {
    fn from(err: CouchError) -> Self {
        Self::new(ErrorCode::Internal, &err.to_string())
    }
}

impl From<hold::error::Error> for Error {
    fn from(err: hold::error::Error) -> Self {
        log::error!("{}", &err);
        match err {
            hold::error::Error::IDNotFound { .. } => Self::from(ErrorCode::BlobUnknown),
            hold::error::Error::ProviderError { .. } => Self::from(ErrorCode::Internal),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct ErrorBody {
    errors: Vec<Error>,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self.code {
            ErrorCode::BlobUnknown
            | ErrorCode::BlobUploadUnknown
            | ErrorCode::ManifestUnknown
            | ErrorCode::ManifestBlobUnknown
            | ErrorCode::NotFound
            | ErrorCode::NameUnknown => StatusCode::NOT_FOUND,
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::Denied => StatusCode::FORBIDDEN,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = ErrorBody {
            errors: vec![self.clone()],
        };

        let mut res = HttpResponse::build(self.status_code());
        let mut accept = Vec::new();
        accept.extend(MediaType::ImageIndex.compatible_types());
        accept.extend(MediaType::ImageManifest.compatible_types());
        res.header(header::ACCEPT, accept.join(","));
        res.json(body)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// This error may be returned when a blob is unknown to the registry in a specified repository. This can be returned with a standard get or if a manifest references an unknown layer during upload.
    BlobUnknown,
    /// The blob upload encountered an error and can no longer proceed.
    BlobUploadInvalid,
    /// If a blob upload has been cancelled or was never started, this error code may be returned.
    BlobUploadUnknown,
    /// When a blob is uploaded, the registry will check that the content matches the digest provided by the client. The error may include a detail structure with the key “digest”, including the invalid digest string. This error may also be returned when a manifest includes an invalid layer digest.
    DigestInvalid,
    /// This error may be returned when a manifest blob is unknown to the registry.
    ManifestBlobUnknown,
    /// During upload, manifests undergo several checks ensuring validity. If those checks fail, this error may be returned, unless a more specific error is included. The detail will contain information the failed validation.
    ManifestInvalid,
    /// This error is returned when the manifest, identified by name and tag is unknown to the repository.
    ManifestUnknown,
    /// During manifest upload, if the manifest fails signature verification, this error will be returned.
    ManifestUnverified,
    /// Invalid repository name encountered either during manifest validation or any API operation.
    NameInvalid,
    /// This is returned if the name used during an operation is unknown to the registry.
    NameUnknown,
    /// When a layer is uploaded, the provided size will be checked against the uploaded content. If they do not match, this error will be returned.
    SizeInvalid,
    /// During a manifest upload, if the tag in the manifest does not match the uri tag, this error will be returned.
    TagInvalid,
    /// The access controller was unable to authenticate the client. Often this will be accompanied by a Www-Authenticate HTTP response header indicating how to authenticate.
    Unauthorized,
    /// The access controller denied access for the operation on a resource.
    Denied,
    /// The operation was unsupported due to a missing implementation or invalid set of parameters.
    Unsupported,
    /// The media type was unsupported due to a missing implementation
    MediaTypeUnsupported,
    ///
    /// Not found
    NotFound,
    /// Internal server error.
    Internal,
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let status = err.status_code();
        ApiError::new(status, err.to_string())
    }
}
