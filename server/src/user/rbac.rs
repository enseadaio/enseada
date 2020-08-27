use std::sync::Arc;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, put};
use futures::future::BoxFuture;
use futures::FutureExt;
use serde::Deserialize;
use tokio::sync::RwLock;

use api::rbac::v1beta1::{PermissionModel, RoleModel, UserCapabilities};
use enseada::couchdb::repository::{Entity, Repository};
use enseada::guid::Guid;
use enseada::pagination::Page;
use oauth::scope::Scope;
use rbac::role::Role;
use rbac::{Enforcer, Rule};
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
    Scope::from(vec!["users:manage", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "add_permission")?;

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
    Scope::from(vec!["users:manage", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "add_permission")?;

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

#[get("/api/v1beta1/users/me/capabilities")]
pub async fn list_capabilities(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> ApiResult<Json<UserCapabilities>> {
    Scope::from("profile").matches_exactly(&scope)?;

    let mut permissions =
        recursively_collect_permissions(&enforcer, current_user.id(), 0, Vec::new()).await?;

    let roles = recursively_collect_roles(&enforcer, current_user.id(), 0, Vec::new()).await?;
    for role in &roles {
        permissions = recursively_collect_permissions(&enforcer, role.id(), 0, permissions).await?;
    }

    permissions.dedup_by(|a, b| a.object() == b.object() && a.action() == b.action());

    let permissions = permissions
        .into_iter()
        .map(map_owned_rule_to_perm)
        .map(|mut perm| {
            // The subject is always the current user
            // so in this context it does not matter if a permission comes
            // from a role or from the user itself
            perm.subject = None;
            perm
        })
        .collect();
    Ok(Json(UserCapabilities {
        permissions,
        roles: roles
            .into_iter()
            .map(|role| role.name().to_string())
            .collect(),
    }))
}

pub fn recursively_collect_permissions<'a>(
    enforcer: &'a RwLock<Enforcer>,
    sub: &'a Guid,
    offset: usize,
    mut acc: Vec<Rule>,
) -> BoxFuture<'a, ApiResult<Vec<Rule>>> {
    async move {
        let enf = enforcer.read().await;
        let page = enf.list_principal_permissions(sub, 100, offset).await?;
        let last = page.is_last();
        acc.extend(page);
        if last {
            Ok(acc)
        } else {
            recursively_collect_permissions(enforcer, sub, offset + 100, acc).await
        }
    }
    .boxed()
}

pub fn recursively_collect_roles<'a>(
    enforcer: &'a RwLock<Enforcer>,
    sub: &'a Guid,
    offset: usize,
    mut acc: Vec<Role>,
) -> BoxFuture<'a, ApiResult<Vec<Role>>> {
    async move {
        let enf = enforcer.read().await;
        let page = enf.list_principal_roles(sub, 100, offset).await?;
        let last = page.is_last();
        acc.extend(page);
        if last {
            Ok(acc)
        } else {
            recursively_collect_roles(enforcer, sub, offset + 100, acc).await
        }
    }
    .boxed()
}
