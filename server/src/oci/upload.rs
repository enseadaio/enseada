use actix_web::http::HeaderMap;
use actix_web::web::{Bytes, Data, Path, Query};
use actix_web::{delete, get, patch, post, put, HttpRequest, HttpResponse};
use serde::Deserialize;

use enseada::couchdb::repository::{Entity, Repository};
use oci::digest::Digest;
use oci::entity::{Blob, Repo, UploadChunk};
use oci::error::{Error, ErrorCode};
use oci::header;
use oci::service::{BlobService, RepoService, UploadService};

use crate::oci::{RepoPath, Result};

#[derive(Debug, Deserialize)]
pub struct DigestParam {
    pub digest: Digest,
}

#[post("/{group}/{name}/blobs/uploads/")]
pub async fn start(
    uploads: Data<UploadService>,
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    // enforcer: Data<RwLock<Enforcer>>,
    // scope: OAuthScope,
    // current_user: CurrentUser,
    path: Path<RepoPath>,
    query: Option<Query<DigestParam>>,
    body: Bytes,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let group = &path.group;
    let name = &path.name;
    let repo = &repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let upload = uploads.start_upload(repo).await?;
    let upload_id = upload.id().id();
    match query {
        Some(query) => {
            let digest = &query.digest;
            let chunk = chunk_from_request(req.headers().into(), body)?;
            // TODO: check digest matches chunk
            uploads
                .complete_upload(upload_id, digest, Some(chunk))
                .await?;
            let blob = Blob::new(digest.clone());
            blobs.save(blob).await?;
            Ok(HttpResponse::Created()
                .header(
                    http::header::LOCATION,
                    format!("/v2/{}/blobs/{}", repo.full_name(), digest),
                )
                .header(header::BLOB_UPLOAD_ID, upload_id)
                .finish())
        }
        None => Ok(HttpResponse::Accepted()
            .header(
                http::header::LOCATION,
                format!("/v2/{}/blobs/uploads/{}", repo.full_name(), upload_id),
            )
            .header(http::header::RANGE, "0-0")
            .header(header::BLOB_UPLOAD_ID, upload_id)
            .finish()),
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadPath {
    upload_id: String,
}

#[get("/{group}/{name}/blobs/uploads/{upload_id}")]
pub async fn get(
    uploads: Data<UploadService>,
    repos: Data<RepoService>,
    // enforcer: Data<RwLock<Enforcer>>,
    // scope: OAuthScope,
    // current_user: CurrentUser,
    repo: Path<RepoPath>,
    upload: Path<UploadPath>,
) -> Result<HttpResponse> {
    let group = &repo.group;
    let name = &repo.name;
    let upload_id = &upload.upload_id;
    log::debug!("looking for repo {}/{}", group, name);
    repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    log::debug!("looking for upload {}", upload_id);
    let upload = uploads
        .find(upload_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

    Ok(HttpResponse::NoContent()
        .header(http::header::RANGE, format!("0-{}", upload.latest_offset()))
        .header(header::BLOB_UPLOAD_ID, upload.id().id())
        .finish())
}

#[patch("/{group}/{name}/blobs/uploads/{upload_id}")]
pub async fn push(
    uploads: Data<UploadService>,
    repos: Data<RepoService>,
    // enforcer: Data<RwLock<Enforcer>>,
    // scope: OAuthScope,
    // current_user: CurrentUser,
    repo: Path<RepoPath>,
    upload: Path<UploadPath>,
    body: Bytes,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let group = &repo.group;
    let name = &repo.name;
    let upload_id = upload.upload_id.as_str();
    log::debug!("looking for repo {}/{}", group, name);
    let repo = repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    log::debug!("building chunk");
    let chunk = chunk_from_request(req.headers(), body)?;

    log::debug!("pushing chunk");
    let upload = uploads.push_chunk(upload_id, chunk).await?;

    Ok(HttpResponse::Accepted()
        .header(
            http::header::LOCATION,
            format!("/v2/{}/blobs/uploads/{}", repo.full_name(), upload_id),
        )
        .header(http::header::RANGE, format!("0-{}", upload.latest_offset()))
        .header(header::BLOB_UPLOAD_ID, upload_id)
        .finish())
}

#[derive(Debug, Deserialize)]
pub struct CompletePath {
    group: String,
    name: String,
    upload_id: String,
}

#[put("/{group}/{name}/blobs/uploads/{upload_id}")]
pub async fn complete(
    uploads: Data<UploadService>,
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    // enforcer: Data<RwLock<Enforcer>>,
    // scope: OAuthScope,
    // current_user: CurrentUser,
    path: Path<CompletePath>,
    digest: Query<DigestParam>,
    body: Option<Bytes>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let group = &path.group;
    let name = &path.name;
    let upload_id = path.upload_id.as_str();
    let digest = &digest.digest;

    log::debug!("looking for repo {}/{}", group, name);
    let repo = repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let chunk = match body {
        Some(body) => {
            log::debug!("building chunk");
            Some(chunk_from_request(req.headers(), body)?)
        }
        None => None,
    };

    log::debug!("completing upload");
    // TODO: check digest matches chunk
    let upload = uploads.complete_upload(upload_id, &digest, chunk).await?;
    let digest_s = digest.to_string();
    let blob = Blob::new(digest.clone());
    blobs.save(blob).await?;

    Ok(HttpResponse::Created()
        .header(
            http::header::LOCATION,
            format!("/v2/{}/blobs/{}", repo.full_name(), digest_s),
        )
        .header(
            http::header::CONTENT_RANGE,
            format!("0-{}", upload.latest_offset()),
        )
        .header(header::CONTENT_DIGEST, digest_s)
        .finish())
}

#[delete("/{group}/{name}/blobs/uploads/{upload_id}")]
pub async fn delete(
    uploads: Data<UploadService>,
    repos: Data<RepoService>,
    // enforcer: Data<RwLock<Enforcer>>,
    // scope: OAuthScope,
    // current_user: CurrentUser,
    repo: Path<RepoPath>,
    upload: Path<UploadPath>,
) -> Result<HttpResponse> {
    let group = &repo.group;
    let name = &repo.name;
    let upload_id = upload.upload_id.as_str();

    log::debug!("looking for repo {}/{}", group, name);
    repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    log::debug!("looking for upload {}", upload_id);
    let upload = uploads
        .find(upload_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

    uploads.delete(&upload).await?;

    Ok(HttpResponse::NoContent().finish())
}

fn chunk_from_request(headers: &HeaderMap, body: Bytes) -> Result<UploadChunk> {
    let content_length = headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.parse().ok())
        .unwrap_or_else(|| body.len());

    let content_range = headers.get(http::header::CONTENT_RANGE);
    let (start_range, end_range) = if let Some(hdr) = content_range {
        let value = hdr
            .to_str()
            .map_err(|_| Error::from(ErrorCode::Unsupported))?;
        let range: Vec<&str> = value.split('-').collect();
        let start_range: usize = range.first().unwrap().parse().unwrap();
        let end_range: usize = range.last().unwrap().parse().unwrap();
        (start_range, end_range)
    } else {
        (0, content_length)
    };

    Ok(UploadChunk::new(start_range, end_range, body.to_vec()))
}
