use std::sync::Arc;

use actix_web::web::{Data, Json, Path, Query, ServiceConfig};
use actix_web::{delete, get, post, put};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use api::rbac::v1beta1::{PermissionModel, RoleModel};
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
use crate::user::UsernamePathParam;

pub fn mount(cfg: &mut ServiceConfig) {
    // Roles
    cfg.service(list_roles);
    cfg.service(create_role);
    cfg.service(delete_role);

    // Role Permissions
    cfg.service(list_role_permissions);
    cfg.service(add_role_permission);
    cfg.service(remove_role_permission);
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RoleResponse {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct RolePathParam {
    pub role: String,
}

#[get("/api/v1beta1/roles")]
pub async fn list_roles(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<RoleModel>>> {
    Scope::from("roles").matches_exactly(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("roles"), "read")?;

    let limit = list.limit();
    let offset = list.offset();

    let roles: Page<Role> = enforcer.list(limit, offset).await?;
    Ok(Json(roles.map(map_owned_role)))
}

#[post("/api/v1beta1/roles")]
pub async fn create_role(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    body: Json<RoleModel>,
) -> ApiResult<Json<RoleModel>> {
    Scope::from("roles").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("roles"), "create")?;

    let role = Role::new(&body.role);
    let role = enforcer.save(role).await?;

    Ok(Json(map_owned_role(role)))
}

#[delete("/api/v1beta1/roles/{role}")]
pub async fn delete_role(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RolePathParam>,
) -> ApiResult<Json<RoleModel>> {
    Scope::from("roles").matches_exactly(&scope)?;
    let role = &path.role;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Role::build_guid(role), "manage")?;

    let role = enforcer.get(role).await?;
    enforcer.delete(&role).await?;
    Ok(Json(map_owned_role(role)))
}

#[get("/api/v1beta1/roles/{role}/permissions")]
pub async fn list_role_permissions(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RolePathParam>,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<PermissionModel>>> {
    Scope::from(vec!["roles", "permissions"]).matches_exactly(&scope)?;
    let role = &path.role;
    let enforcer = enforcer.read().await;
    let sub = &Guid::partitioned("role", role);
    enforcer.check(current_user.id(), sub, "read_permissions")?;

    let limit = list.limit();
    let offset = list.offset();

    let rules = enforcer
        .list_principal_permissions(&sub, limit, offset)
        .await?;
    let permissions = rules.map(map_owned_rule_to_perm);
    Ok(Json(permissions))
}

#[post("/api/v1beta1/roles/{role}/permissions")]
pub async fn add_role_permission(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RolePathParam>,
    permission: Json<PermissionModel>,
) -> ApiResult<Json<PermissionModel>> {
    Scope::from(vec!["roles", "permissions"]).matches_exactly(&scope)?;
    let role = &path.role;
    let enforcer = enforcer.read().await;
    let sub = Guid::partitioned("role", role);
    enforcer.check(current_user.id(), &sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer
        .add_permission(sub, permission.object.clone(), &permission.action)
        .await?;

    Ok(permission)
}

#[delete("/api/v1beta1/roles/{role}/permissions")]
pub async fn remove_role_permission(
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<RolePathParam>,
    permission: Json<PermissionModel>,
) -> ApiResult<Json<PermissionModel>> {
    Scope::from(vec!["roles", "permissions"]).matches_exactly(&scope)?;
    let role = &path.role;
    let enforcer = enforcer.read().await;
    let sub = &Guid::partitioned("role", role);
    enforcer.check(current_user.id(), sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer
        .remove_permission(sub, permission.object.clone(), &permission.action)
        .await?;

    Ok(permission)
}

pub fn map_owned_role(role: Role) -> RoleModel {
    RoleModel {
        role: role.name().to_string(),
    }
}

pub fn map_owned_rule_to_perm(rule: Rule) -> PermissionModel {
    PermissionModel {
        subject: Some(rule.subject().clone()),
        object: rule.object().clone(),
        action: rule.action().to_string(),
    }
}
