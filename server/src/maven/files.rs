use actix_web::web::{Data, Path};
use actix_web::{get, HttpResponse};
use serde::Deserialize;

use enseada::error::Error;
use maven::file::parse_file_path;
use maven::service::RepoService;

use crate::http::error::ApiError;
use crate::http::ApiResult;

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
