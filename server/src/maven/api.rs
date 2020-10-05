use std::sync::Arc;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use enseada::couchdb::repository::{Entity, Repository};
use enseada::guid::Guid;
use enseada::pagination::Page;
use maven::entity::Repo;
use maven::service::RepoService;
use oauth::scope::Scope;
use rbac::Enforcer;

use crate::http::error::ApiError;
use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::user::CurrentUser;
use crate::http::{ApiResult, PaginationQuery};

#[derive(Debug, Deserialize)]
pub struct RepoPath {
    group_id: String,
    artifact_id: String,
}

#[derive(Debug, Serialize)]
pub struct RepoResponse {
    group_id: String,
    artifact_id: String,
    public: bool,
}

impl From<&Repo> for RepoResponse {
    fn from(repo: &Repo) -> Self {
        Self {
            group_id: repo.group_id().to_string(),
            artifact_id: repo.artifact_id().to_string(),
            public: repo.is_public(),
        }
    }
}

impl From<Repo> for RepoResponse {
    fn from(repo: Repo) -> Self {
        Self::from(&repo)
    }
}

#[get("/api/maven/v1beta1/repositories")]
pub async fn list_repos(
    service: Data<RepoService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<RepoResponse>>> {
    Scope::from("maven:repos:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("maven_repos"), "read")?;
    let limit = list.limit();
    let offset = list.offset();

    let page = service.list(limit, offset).await?.map(RepoResponse::from);
    Ok(Json(page))
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoPayload {
    group_id: String,
    artifact_id: String,
    #[serde(default)]
    public: bool,
}

#[post("/api/maven/v1beta1/repositories")]
pub async fn create_repo(
    service: Data<RepoService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    body: Json<CreateRepoPayload>,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("maven:repos:manage").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("maven_repos"), "create")?;

    let repo = Repo::new(&body.group_id, &body.artifact_id, body.public);
    let repo = service.save(repo).await?;

    Ok(Json(RepoResponse::from(repo)))
}

#[get("/api/maven/v1beta1/repositories/{group_id}/{artifact_id}")]
pub async fn get_repo(
    service: Data<RepoService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("maven:repos:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let group_id = &path.group_id;
    let artifact_id = &path.artifact_id;
    let id = &Repo::build_id(group_id, artifact_id);
    log::warn!("{}", Repo::build_guid(id));
    enforcer.check(current_user.id(), &Repo::build_guid(id), "read")?;

    let repo = service
        .find(id)
        .await?
        .ok_or_else(|| ApiError::not_found(&format!("OCI repository '{}' not found", id)))?;

    Ok(Json(RepoResponse::from(repo)))
}

#[get("/api/maven/v1beta1/repositories/{group_id}/{artifact_id}/files")]
pub async fn get_repo_files(
    service: Data<RepoService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
) -> ApiResult<Json<Vec<String>>> {
    Scope::from("maven:repos:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let group_id = &path.group_id;
    let artifact_id = &path.artifact_id;
    let id = &Repo::build_id(group_id, artifact_id);
    log::warn!("{}", Repo::build_guid(id));
    enforcer.check(current_user.id(), &Repo::build_guid(id), "read")?;

    let repo = service
        .find(id)
        .await?
        .ok_or_else(|| ApiError::not_found(&format!("OCI repository '{}' not found", id)))?;

    Ok(Json(Vec::from(repo.files())))
}

#[delete("/api/maven/v1beta1/repositories/{group_id}/{artifact_id}")]
pub async fn delete_repo(
    service: Data<RepoService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("maven:repos:delete").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let group_id = &path.group_id;
    let artifact_id = &path.artifact_id;
    let id = &Repo::build_id(group_id, artifact_id);
    log::warn!("id: {}", id);
    enforcer.check(current_user.id(), &Repo::build_guid(id), "delete")?;

    let repo = service.find(id).await?.ok_or_else(|| {
        ApiError::not_found(&format!(
            "OCI repository '{}/{}' not found",
            group_id, artifact_id
        ))
    })?;

    log::warn!("deleting repo {}", repo.id());
    service.delete(&repo).await?;

    Ok(Json(RepoResponse::from(repo)))
}
