use std::convert::TryFrom;
use std::sync::Arc;

use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde::Deserialize;
use tokio::sync::RwLock;

use enseada::couchdb::repository::{Entity, Repository};
use oauth::scope::Scope;
use oci::digest::Digest;
use oci::entity::{Manifest, Repo};
use oci::error::{Error, ErrorCode};
use oci::header;
use oci::manifest::ImageManifest;
use oci::mime::MediaType;
use oci::service::{ManifestService, RepoService};
use rbac::Enforcer;

use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::session::TokenSession;
use crate::http::extractor::user::CurrentUser;
use crate::oci::{RepoPath, Result};

#[derive(Debug, Deserialize)]
pub struct ManifestRefParam {
    reference: String,
}

pub async fn get(
    manifests: Data<ManifestService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    reference: Path<ManifestRefParam>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> Result<HttpResponse> {
    Scope::from("oci:image:pull").matches(&scope)?;
    let group = &repo.group;
    let name = &repo.name;
    let reference = &reference.reference;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:pull")?;

    log::debug!("looking for repo {}/{}", group, name);
    repos
        .find(&repo_id)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let manifest = manifests
        .find_by_ref(reference)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::ManifestUnknown))?;

    let manifest = manifest.into_inner();
    Ok(HttpResponse::Ok()
        .header(
            http::header::CONTENT_TYPE,
            MediaType::ImageManifest.to_string(),
        )
        .header(header::CONTENT_DIGEST, manifest.digest().to_string())
        .json(manifest))
}

pub async fn put(
    manifests: Data<ManifestService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    reference: Path<ManifestRefParam>,
    body: Json<ImageManifest>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> Result<HttpResponse> {
    Scope::from("oci:image:push").matches(&scope)?;
    let group = &repo.group;
    let name = &repo.name;
    let reference = &reference.reference;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:push")?;

    log::debug!("looking for repo {}/{}", group, name);
    let repo = repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let manifest = Manifest::new(reference, group, name, body.into_inner());
    let manifest = manifests.save(manifest).await?;

    log::debug!("Checking if ref '{}' is a tag", reference);
    if Digest::try_from(reference).is_err() {
        log::debug!("Ref '{}' is indeed a tag", reference);
        // Reference is a tag
        let mut repo = repo;
        repo.push_tag(reference.clone());
        repos.save(repo).await?;
    }

    let manifest = manifest.into_inner();

    Ok(HttpResponse::Created()
        .header(
            http::header::LOCATION,
            format!("/{}/{}/manifests/{}", group, name, reference),
        )
        .header(header::CONTENT_DIGEST, manifest.digest().to_string())
        .finish())
}

pub async fn delete(
    manifests: Data<ManifestService>,
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    reference: Path<ManifestRefParam>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> Result<HttpResponse> {
    Scope::from("oci:image:delete").matches(&scope)?;
    let group = &repo.group;
    let name = &repo.name;
    let reference = &reference.reference;
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

    let manifest = manifests
        .find_by_ref(reference)
        .await?
        .ok_or_else(|| Error::from(ErrorCode::ManifestUnknown))?;

    manifests.delete(&manifest).await?;

    Ok(HttpResponse::Accepted().finish())
}
