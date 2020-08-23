use std::ops::Deref;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, put};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use api::pat::v1beta1::{
    CreatedPersonalAccessToken, PersonalAccessTokenModel, PersonalAccessTokenPost,
};
use enseada::couchdb::repository::{Entity, Repository};
use enseada::expiration::Expiration;
use enseada::guid::Guid;
use enseada::pagination::Page;
use oauth::error::ErrorKind;
use oauth::handler::{OAuthHandler, TokenIntrospectionHandler};
use oauth::persistence::pat::PersonalAccessToken;
use oauth::persistence::token::AccessTokenEntity;
use oauth::persistence::CouchStorage;
use oauth::scope::Scope;
use oauth::session::Session;
use oauth::storage::TokenStorage;
use oauth::token::{AccessToken, Token};
use oauth::{CouchOAuthHandler, Expirable};
use rbac::Enforcer;

use crate::http::error::ApiError;
use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::session::TokenSession;
use crate::http::extractor::user::CurrentUser;
use crate::http::{ApiResult, PaginationQuery};

#[get("/api/oauth/v1beta1/pats")]
pub async fn list(
    storage: Data<CouchStorage>,
    scope: OAuthScope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<PersonalAccessTokenModel>>> {
    Scope::from("pats:read").matches(&scope)?;
    let user_id = current_user.id();

    let limit = list.limit();
    let offset = list.offset();

    let page = storage
        .find_all(
            limit,
            offset,
            serde_json::json!({ "session.user_id": user_id }),
        )
        .await?
        .map(map_owned_pat);

    Ok(Json(page))
}

#[post("/api/oauth/v1beta1/pats")]
pub async fn create(
    handler: Data<CouchOAuthHandler>,
    storage: Data<CouchStorage>,
    enforcer: Data<RwLock<Enforcer>>,
    session: TokenSession,
    scope: OAuthScope,
    current_user: CurrentUser,
    body: Json<PersonalAccessTokenPost>,
) -> ApiResult<Json<CreatedPersonalAccessToken>> {
    Scope::from("pats:manage").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let label = body.label.clone();
    let scope = body.scope.clone();
    let expiration = body.expiration.unwrap_or_else(|| chrono::MAX_DATETIME);
    if expiration <= Utc::now() {
        return Err(ApiError::invalid(format!(
            "Expiration must be in the future, was '{}'",
            expiration
        )));
    }

    log::debug!("creating new PAT '{}'", label);
    let (access_token, sig) = handler.generate_token_with_sig()?;
    let access_token_value = access_token.to_string();
    let sig = sig.to_string();
    let mut session = session.deref().clone();
    session.set_scope(scope);

    log::debug!("creating access token");
    let at = AccessToken::new(access_token, session, expiration);
    log::debug!("storing access token");
    let at = storage.store_token(&sig, at).await?;
    log::debug!("access token stored");
    let pat = PersonalAccessToken::new(label, &sig, at.session().clone(), at.expiration());
    log::debug!("saving PAT");
    let pat = storage.save(pat).await?;
    log::debug!("PAT saved");
    let pat_id = pat.id();

    let user_id = current_user.id();
    log::debug!("adding permission '*' to {} for user {}", pat_id, user_id);
    enforcer
        .add_permission(user_id.clone(), pat_id.clone(), "*")
        .await?;

    Ok(Json(CreatedPersonalAccessToken {
        access_token: access_token_value,
        pat: map_pat(&pat),
    }))
}

#[derive(Debug, Deserialize)]
pub struct PersonalAccessTokenPathParam {
    pub pat_id: String,
}

#[get("/api/oauth/v1beta1/pats/{pat_id}")]
pub async fn get(
    storage: Data<CouchStorage>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<PersonalAccessTokenPathParam>,
) -> ApiResult<Json<PersonalAccessTokenModel>> {
    Scope::from("pats:read").matches(&scope)?;
    let pat_id = &path.pat_id;
    let client = storage.find(pat_id).await?;

    let pat = client.ok_or_else(|| {
        ApiError::not_found(&format!("Personal Access Token '{}' not found", pat_id))
    })?;
    check_pat_owner(&pat, &current_user)?;
    Ok(Json(map_pat(&pat)))
}

#[delete("/api/oauth/v1beta1/pats/{pat_id}")]
pub async fn delete(
    storage: Data<CouchStorage>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<PersonalAccessTokenPathParam>,
) -> ApiResult<Json<PersonalAccessTokenModel>> {
    Scope::from("pats:manage").matches(&scope)?;
    let pat_id = &path.pat_id;

    let pat = storage
        .find(pat_id)
        .await?
        .ok_or_else(|| ApiError::not_found(&format!("PAT '{}' not found", pat_id)))?;

    check_pat_owner(&pat, &current_user)?;

    log::debug!("deleting related access token");
    if let Some(err) = TokenStorage::<AccessToken>::revoke_token(storage.get_ref(), pat_id)
        .await
        .err()
    {
        if err.kind() != &ErrorKind::InvalidRequest {
            return Err(ApiError::from(err));
        } else {
            log::debug!("no access token found associated with PAT '{}'", pat_id)
        }
    }

    log::debug!("deleting PAT");
    storage.delete(&pat).await?;
    log::debug!("PAT deleted");

    Ok(Json(map_pat(&pat)))
}

fn check_pat_owner(pat: &PersonalAccessToken, current_user: &CurrentUser) -> ApiResult<()> {
    if pat.session().user_id() != Some(&current_user.id().to_string()) {
        Err(ApiError::not_found(&format!(
            "PAT '{}' not found",
            pat.id().id()
        )))
    } else {
        Ok(())
    }
}

fn map_pat(pat: &PersonalAccessToken) -> PersonalAccessTokenModel {
    let session = pat.session();
    PersonalAccessTokenModel {
        id: pat.id().id().to_string(),
        label: pat.label().to_string(),
        client_id: session.client_id().to_string(),
        scope: session.scope().clone(),
        user_id: session.user_id().map(str::to_string),
        expiration: pat.expiration().into(),
        revoked_at: pat.revoked_at(),
    }
}

#[inline]
fn map_owned_pat(pat: PersonalAccessToken) -> PersonalAccessTokenModel {
    map_pat(&pat)
}
