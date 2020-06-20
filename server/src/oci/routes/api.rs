use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use enseada::guid::Guid;
use enseada::pagination::{Cursor, Page};

use crate::couchdb::repository::{Entity, Repository};
use crate::http::error::ApiError;
use crate::http::extractor::user::CurrentUser;
use crate::http::{ApiResult, PaginationQuery};
use crate::oauth::scope::Scope;
use crate::oci::entity::Repo;
use crate::oci::routes::RepoPath;
use crate::oci::service::RepoService;
use crate::rbac::Enforcer;

#[derive(Debug, Serialize)]
pub struct RepoResponse {
    group: String,
    name: String,
}

impl From<&Repo> for RepoResponse {
    fn from(repo: &Repo) -> Self {
        Self {
            group: repo.group().to_string(),
            name: repo.name().to_string(),
        }
    }
}

impl From<Repo> for RepoResponse {
    fn from(repo: Repo) -> Self {
        Self::from(&repo)
    }
}

#[get("/api/oci/v1beta1/repositories")]
pub async fn list_repos(
    service: Data<RepoService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<RepoResponse>>> {
    Scope::from("oci:repos:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("oci_repos"), "read")?;
    let limit = list.limit();
    let cursor = list.cursor();

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };

    let page = service
        .list(limit, cursor.as_ref())
        .await?
        .map(|repo| RepoResponse::from(repo));
    Ok(Json(page))
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoPayload {
    group: String,
    name: String,
}

#[post("/api/oci/v1beta1/repositories")]
pub async fn create_repo(
    service: Data<RepoService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    body: Json<CreateRepoPayload>,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("oci:repos:manage").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("oci_repos"), "create")?;

    let repo = Repo::new(&body.group, &body.name);
    let repo = service.save(repo).await?;

    Ok(Json(RepoResponse::from(repo)))
}

#[get("/api/oci/v1beta1/repositories/{group}/{name}")]
pub async fn get_repo(
    service: Data<RepoService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("oci:repos:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let group = &path.group;
    let name = &path.name;
    let id = &Repo::build_id(group, name);
    enforcer.check(current_user.id(), &Repo::build_guid(id), "read")?;

    let repo = service.find(id).await?.ok_or_else(|| {
        ApiError::not_found(&format!("OCI repository '{}/{}' not found", group, name))
    })?;

    Ok(Json(RepoResponse::from(repo)))
}

#[delete("/api/oci/v1beta1/repositories/{group}/{name}")]
pub async fn delete_repo(
    service: Data<RepoService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<RepoPath>,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("oci:repos:delete").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let group = &path.group;
    let name = &path.name;
    let id = &Repo::build_id(group, name);
    enforcer.check(current_user.id(), &Repo::build_guid(id), "delete")?;

    let repo = service.find(id).await?.ok_or_else(|| {
        ApiError::not_found(&format!("OCI repository '{}/{}' not found", group, name))
    })?;

    service.delete(&repo).await?;

    Ok(Json(RepoResponse::from(repo)))
}
