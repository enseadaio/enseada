use std::sync::Arc;

use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use actix_web::{delete, get, head};
use tokio::sync::RwLock;

use enseada::couchdb::repository::{Entity, Repository};
use oauth::scope::Scope;
use oci::entity::Repo;
use oci::error::{Error, ErrorCode};
use oci::header;
use oci::service::{BlobService, RepoService};
use rbac::Enforcer;

use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::user::CurrentUser;
use crate::oci::upload::DigestParam;
use crate::oci::{RepoPath, Result};

#[get("/{group}/{name}/blobs/{digest}")]
pub async fn get(
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    digest: Path<DigestParam>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> Result<HttpResponse> {
    Scope::from("oci:image:pull").matches(&scope)?;
    let group = &repo.group;
    let name = &repo.name;
    let digest = &digest.digest;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:pull")?;

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

#[head("/{group}/{name}/blobs/{digest}")]
pub async fn head(
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    digest: Path<DigestParam>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> Result<HttpResponse> {
    Scope::from("oci:image:pull").matches(&scope)?;
    let group = &repo.group;
    let name = &repo.name;
    let digest = &digest.digest;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:pull")?;

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

    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DIGEST, digest_s)
        .finish())
}

#[delete("/{group}/{name}/blobs/{digest}")]
pub async fn delete(
    blobs: Data<BlobService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    digest: Path<DigestParam>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> Result<HttpResponse> {
    Scope::from("oci:image:delete").matches(&scope)?;
    let group = &repo.group;
    let name = &repo.name;
    let digest = &digest.digest;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(
        current_user.id(),
        &Repo::build_guid(&repo_id),
        "image:delete",
    )?;

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
