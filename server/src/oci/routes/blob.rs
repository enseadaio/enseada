use actix_web::web::{Data, Path};
use actix_web::{delete, get, HttpResponse};

use crate::couchdb::repository::Repository;
use crate::oci::entity::Repo;
use crate::oci::error::{Error, ErrorCode};
use crate::oci::routes::upload::DigestParam;
use crate::oci::routes::RepoPath;
use crate::oci::service::{BlobService, RepoService};
use crate::oci::{header, Result};

#[get("/{group}/{name}/blobs/{digest}")]
pub async fn get(
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    digest: Path<DigestParam>,
) -> Result<HttpResponse> {
    let group = &repo.group;
    let name = &repo.name;
    let digest = &digest.digest;

    log::debug!("looking for repo {}/{}", group, name);
    repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    log::debug!("looking for blob {}", digest);
    let digest_s = digest.to_string();
    blobs
        .find(&digest_s)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::BlobUnknown))?;

    let content = blobs.fetch_content(&digest).await?;

    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DIGEST, digest_s)
        .body(content))
}

#[delete("/{group}/{name}/blobs/{digest}")]
pub async fn delete(
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    digest: Path<DigestParam>,
) -> Result<HttpResponse> {
    let group = &repo.group;
    let name = &repo.name;
    let digest = &digest.digest;

    log::debug!("looking for repo {}/{}", group, name);
    repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    log::debug!("looking for blob {}", digest);
    let digest_s = digest.to_string();
    let blob = blobs
        .find(&digest_s)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::BlobUnknown))?;

    blobs.delete_blob(&blob).await?;
    Ok(HttpResponse::Accepted()
        .header(header::CONTENT_DIGEST, digest_s)
        .finish())
}
