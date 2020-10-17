use std::sync::{Arc, Mutex};

use actix_web::http::HeaderMap;
use actix_web::web::{Bytes, Data, Path, Payload, Query};
use actix_web::{delete, get, patch, post, put, HttpRequest, HttpResponse};
use futures::TryStreamExt;
use serde::Deserialize;
use tokio::sync::RwLock;

use enseada::couchdb::repository::{Entity, Repository};
use oauth::scope::Scope;
use oci::digest::Digest;
use oci::entity::{Blob, Repo, UploadChunk};
use oci::error::{Error, ErrorCode};
use oci::header;
use oci::service::{BlobService, RepoService, UploadService};
use rbac::Enforcer;

use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::user::CurrentUser;
use crate::oci::{RepoPath, Result};
use std::io;

#[derive(Debug, Deserialize)]
pub struct DigestParam {
    pub digest: Digest,
}

#[post("/{group}/{name}/blobs/uploads")]
pub async fn start(
    uploads: Data<UploadService>,
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
    query: Option<Query<DigestParam>>,
    body: Bytes,
    req: HttpRequest,
) -> Result<HttpResponse> {
    Scope::from("oci:image:push").matches(&scope)?;
    let group = &path.group;
    let name = &path.name;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:push")?;
    let repo = &repos
        .find(&repo_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let upload = uploads.start_upload(repo).await?;
    let upload_id = upload.id().id().to_string();
    match query {
        Some(query) => {
            let digest = &query.digest;
            let chunk = chunk_from_request(req.headers(), body.len())?;
            // TODO: check digest matches chunk
            uploads
                .complete_upload(upload, digest, Some((chunk, body)))
                .await?;
            let blob = Blob::new(digest.clone(), group, name);
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
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
    upload: Path<UploadPath>,
) -> Result<HttpResponse> {
    Scope::from("oci:image:push").matches(&scope)?;
    let group = &path.group;
    let name = &path.name;
    let upload_id = &upload.upload_id;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:push")?;
    log::debug!("looking for repo {}/{}", group, name);
    repos
        .find(&repo_id)
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
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
    upload: Path<UploadPath>,
    body: Bytes,
    req: HttpRequest,
) -> Result<HttpResponse> {
    Scope::from("oci:image:push").matches(&scope)?;
    let group = &path.group;
    let name = &path.name;
    let upload_id = &upload.upload_id;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:push")?;
    log::debug!("looking for repo {}/{}", group, name);
    let repo = repos
        .find(&repo_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let upload = uploads
        .find(upload_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

    log::debug!("building chunk");
    let chunk = chunk_from_request(req.headers(), body.len())?;

    log::debug!("pushing chunk");
    let upload = uploads.push_chunk(upload, chunk, body).await?;

    Ok(HttpResponse::Accepted()
        .header(
            http::header::LOCATION,
            format!("/v2/{}/blobs/uploads/{}", repo.full_name(), upload_id),
        )
        .header(http::header::RANGE, format!("0-{}", upload.latest_offset()))
        .header(header::BLOB_UPLOAD_ID, upload_id.as_str())
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
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<CompletePath>,
    digest: Query<DigestParam>,
    body: Option<Bytes>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    Scope::from("oci:image:push").matches(&scope)?;
    let group = &path.group;
    let name = &path.name;
    let upload_id = &path.upload_id;
    let digest = &digest.digest;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:push")?;

    log::debug!("looking for repo {}/{}", group, name);
    let repo = repos
        .find(&repo_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let upload = uploads
        .find(upload_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown))?;

    let chunk = match body {
        Some(body) => {
            log::debug!("building chunk");
            Some((chunk_from_request(req.headers(), body.len())?, body))
        }
        None => None,
    };

    log::debug!("completing upload");
    // TODO: check digest matches chunk
    let upload = uploads.complete_upload(upload, &digest, chunk).await?;
    let digest_s = digest.to_string();
    let blob = Blob::new(digest.clone(), group, name);
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
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
    upload: Path<UploadPath>,
) -> Result<HttpResponse> {
    Scope::from("oci:image:push").matches(&scope)?;
    let group = &path.group;
    let name = &path.name;
    let upload_id = &upload.upload_id;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:push")?;

    log::debug!("looking for repo {}/{}", group, name);
    repos
        .find(&repo_id)
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

fn chunk_from_request(headers: &HeaderMap, size: usize) -> Result<UploadChunk> {
    let content_length = headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.parse().ok())
        .unwrap_or(size);

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

    Ok(UploadChunk::new(start_range, end_range))
}
