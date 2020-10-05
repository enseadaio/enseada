use actix_web::web::{Bytes, Data, Path};
use actix_web::{get, put, HttpResponse};
use serde::Deserialize;

use enseada::error::Error;
use maven::file::{parse_file_path, File};
use maven::service::RepoService;

use crate::http::error::ApiError;
use crate::http::ApiResult;
use http::StatusCode;

#[derive(Debug, Deserialize)]
pub struct MavenPath {
    tail: String,
}

impl MavenPath {
    pub fn segments(&self) -> Vec<&str> {
        self.tail.split('/').collect()
    }
}

#[get("/maven/{tail:.*}")]
pub async fn get(repos: Data<RepoService>, path: Path<MavenPath>) -> ApiResult<HttpResponse> {
    let location = &path.tail;
    let file_pointer = parse_file_path(location)
        .ok_or_else(|| ApiError::invalid(format!("{} is not a valid Maven path", location)))?;

    let repo = repos.find_by_location(file_pointer.prefix()).await?;
    let repo = match repo {
        Some(repo) => repo,
        None => Err(Error::not_found("Maven repository", file_pointer.prefix()))?,
    };

    if repo.is_private() {
        // TODO(matteojoliveau): authc/authz
    }

    let file = repos
        .get_file(&repo, file_pointer.version(), file_pointer.filename())
        .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .streaming(file.into_byte_stream()))
}

#[put("/maven/{tail:.*}")]
pub async fn put(
    repos: Data<RepoService>,
    path: Path<MavenPath>,
    body: Bytes,
) -> ApiResult<HttpResponse> {
    let location = &path.tail;
    let file_pointer = parse_file_path(location)
        .ok_or_else(|| ApiError::invalid(format!("{} is not a valid Maven path", location)))?;

    let repo = repos.find_by_location(file_pointer.prefix()).await?;
    let repo = match repo {
        Some(repo) => repo,
        None => Err(Error::not_found("Maven repository", file_pointer.prefix()))?,
    };

    if repo.is_private() {
        // TODO(matteojoliveau): authc/authz
    }

    let filename = file_pointer.filename();
    if let Some(version) = file_pointer.version() {
        let file_exists = repos.is_file_present(&repo, version, filename).await?;
        if !version.is_snapshot() && file_exists {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                format!(
                    "could not update file {}, version {} is immutable",
                    filename, version
                ),
            ));
        }
    }

    let file = File::from_bytes(file_pointer.version(), filename, body);
    repos.store_file(&repo, file).await?;

    Ok(HttpResponse::Accepted().finish())
}
