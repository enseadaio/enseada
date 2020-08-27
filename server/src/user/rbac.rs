use std::sync::Arc;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, put};
use serde::Deserialize;
use tokio::sync::RwLock;

use api::rbac::v1beta1::{PermissionModel, RoleModel};
use enseada::couchdb::repository::{Entity, Repository};
use enseada::guid::Guid;
use enseada::pagination::Page;
use oauth::scope::Scope;
use rbac::Enforcer;
use users::UserService;

use crate::http::error::ApiError;
use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::user::CurrentUser;
use crate::http::{ApiResult, PaginationQuery};
use crate::rbac::map_owned_rule_to_perm;
use crate::user::UsernamePathParam;

#[get("/api/v1beta1/users/{username}/permissions")]
pub async fn list_permissions(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<PermissionModel>>> {
    Scope::from(vec!["users:read", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "read_permissions")?;

    let limit = list.limit();
    let offset = list.offset();

    let page = enforcer
        .list_principal_permissions(&sub, limit, offset)
        .await?;
    let permissions = page.map(map_owned_rule_to_perm);
    Ok(Json(permissions))
}

#[post("/api/v1beta1/users/{username}/permissions")]
pub async fn add_permission(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    permission: Json<PermissionModel>,
) -> ApiResult<Json<PermissionModel>> {
    Scope::from(vec!["users:read", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer
        .add_permission(sub, permission.object.clone(), &permission.action)
        .await?;

    Ok(permission)
}

#[delete("/api/v1beta1/users/{username}/permissions")]
pub async fn remove_permission(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    permission: Json<PermissionModel>,
) -> ApiResult<Json<PermissionModel>> {
    Scope::from(vec!["users:read", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer
        .remove_permission(sub, permission.object.clone(), &permission.action)
        .await?;

    Ok(permission)
}

#[get("/api/v1beta1/users/{username}/roles")]
pub async fn list_roles(
    service: Data<UserService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<String>>> {
    Scope::from(vec!["users:read", "roles"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "read_roles")?;

    if service.find(username).await?.is_none() {
        return Err(ApiError::not_found(format!("User {} not found", username)));
    }

    let limit = list.limit();
    let offset = list.offset();

    let page = enforcer.list_principal_roles(&sub, limit, offset).await?;
    Ok(Json(page.map(|role| role.name().to_string())))
}

#[derive(Debug, Deserialize)]
pub struct UserRolesPathParams {
    username: String,
    role: String,
}

#[put("/api/v1beta1/users/{username}/roles/{role}")]
pub async fn add_role(
    service: Data<UserService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UserRolesPathParams>,
) -> ApiResult<Json<RoleModel>> {
    Scope::from(vec!["users:manage", "roles"]).matches_exactly(&scope)?;
    let enforcer = enforcer.read().await;
    let username = &path.username;
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "manage_roles")?;

    if service.find(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    let role = &path.role;
    enforcer.add_role_to_principal(sub.clone(), role).await?;
    Ok(Json(RoleModel { role: role.clone() }))
}

#[delete("/api/v1beta1/users/{username}/roles/{role}")]
pub async fn remove_role(
    service: Data<UserService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UserRolesPathParams>,
) -> ApiResult<Json<RoleModel>> {
    Scope::from(vec!["users:manage", "roles"]).matches_exactly(&scope)?;
    let enforcer = enforcer.read().await;
    let username = &path.username;
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "manage_roles")?;

    if service.find(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    let role = &path.role;
    enforcer.remove_role_from_principal(sub, role).await?;
    Ok(Json(RoleModel { role: role.clone() }))
}
