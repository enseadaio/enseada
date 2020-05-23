use actix_web::web::{Data, Json, Path, Query};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::containers::name::Name;
use crate::containers::repo::{Repo, RepoService};
use crate::guid::Guid;
use crate::http::error::ApiError;
use crate::http::extractor::scope::Scope;
use crate::http::extractor::user::CurrentUser;
use crate::http::handler::{ApiResult, PaginationQuery};
use crate::pagination::{Cursor, Page};
use crate::rbac::Enforcer;
use crate::responses;

#[derive(Debug, Serialize)]
pub struct RepoResponse {
    id: String,
    group: String,
    name: String,
}

impl From<Repo> for RepoResponse {
    fn from(repo: Repo) -> Self {
        Self::from(&repo)
    }
}

impl From<&Repo> for RepoResponse {
    fn from(repo: &Repo) -> Self {
        RepoResponse {
            id: repo.id().id().to_string(),
            group: repo.name().group().to_string(),
            name: repo.name().name().to_string(),
        }
    }
}

pub async fn list(
    service: Data<RepoService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<RepoResponse>>> {
    Scope::from("oci:repos:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("oci:repos"), "read")?;
    let limit = list.limit();
    let cursor = list.cursor();

    log::info!("Listing users with limit {} and cursor {:?}", &limit, &cursor);

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };
    let page = service.list_repos(limit, cursor.as_ref()).await?
        .map(|repo| RepoResponse::from(repo));
    Ok(Json(page))
}

#[derive(Debug, Deserialize)]
pub struct NewRepo {
    pub group: String,
    pub name: String,
}

pub async fn create(
    service: Data<RepoService>,
    data: Json<NewRepo>,
    scope: Scope,
    enforcer: Data<RwLock<Enforcer>>,
    current_user: CurrentUser,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("oci:repos:manage").matches(&scope)?;
    let enf = enforcer.write().await;
    enf.check(current_user.id(), &Guid::simple("oci:repos"), "create")?;

    let name = Name::new(data.group.clone(), data.name.clone());
    let existent = service.find_repo_by_name(&name).await?;
    if existent.is_some() {
        return responses::conflict(format!("a repo with coordinates {} already exists", name));
    }
    let repo = Repo::new(name);
    let repo = service.save_repo(repo).await?;

    enf.add_permission(current_user.id().clone(), repo.id().clone(), "*").await?;

    responses::ok(RepoResponse::from(repo))
}

#[derive(Debug, Deserialize)]
pub struct RepoIDPath {
    repo_id: String,
}

pub async fn get(
    service: Data<RepoService>,
    path: Path<RepoIDPath>,
    scope: Scope,
    enforcer: Data<RwLock<Enforcer>>,
    current_user: CurrentUser,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("oci:repos:read").matches(&scope)?;
    let id = &path.repo_id;
    let enf = enforcer.read().await;
    let guid = &Repo::build_guid(id);
    enf.check(current_user.id(), guid, "read")?;

    let repo = service.find_repo(id).await?;

    match repo {
        Some(repo) => responses::ok(RepoResponse::from(repo)),
        None => responses::not_found(guid)
    }
}

pub async fn delete(
    service: Data<RepoService>,
    path: Path<RepoIDPath>,
    scope: Scope,
    enforcer: Data<RwLock<Enforcer>>,
    current_user: CurrentUser,
) -> ApiResult<Json<RepoResponse>> {
    Scope::from("oci:repos:delete").matches(&scope)?;
    let id = &path.repo_id;
    let enf = enforcer.read().await;
    let guid = &Repo::build_guid(id);
    enf.check(current_user.id(), guid, "delete")?;

    let repo = service.find_repo(id).await?
        .ok_or_else(|| ApiError::NotFound(format!("repo '{}' not found", id)))?;

    service.delete_repo(&repo).await?;

    responses::ok(RepoResponse::from(repo))
}