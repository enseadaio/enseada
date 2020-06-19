use std::collections::HashSet;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, put};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use enseada::guid::Guid;
use enseada::pagination::{Cursor, Page};

use crate::couchdb::repository::Entity;
use crate::http::error::ApiError;
use crate::http::extractor::scope::Scope;
use crate::http::extractor::user::CurrentUser;
use crate::http::{ApiResult, PaginationQuery};
use crate::oauth::client::Client;
use crate::oauth::persistence::client::ClientEntity;
use crate::oauth::persistence::CouchStorage;
use crate::oauth::storage::ClientStorage;
use crate::rbac::Enforcer;

#[derive(Debug, Serialize, PartialEq)]
pub struct ClientResponse {
    pub client_id: String,
    pub kind: String,
    pub allowed_scopes: Scope,
    pub allowed_redirect_uris: HashSet<url::Url>,
}

impl From<Client> for ClientResponse {
    fn from(client: Client) -> Self {
        Self::from(&client)
    }
}

impl From<&Client> for ClientResponse {
    fn from(client: &Client) -> Self {
        ClientResponse {
            client_id: client.client_id().to_string(),
            kind: client.kind().to_string(),
            allowed_scopes: client.allowed_scopes().clone(),
            allowed_redirect_uris: client.allowed_redirect_uris().clone(),
        }
    }
}

#[get("/api/v1beta1/clients")]
pub async fn list_clients(
    storage: Data<CouchStorage>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<ClientResponse>>> {
    Scope::from("clients:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("clients"), "read")?;

    let limit = list.limit();
    let cursor = list.cursor();

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };

    let page = storage
        .list_clients(limit, cursor.as_ref())
        .await?
        .map(|client| ClientResponse::from(client));
    Ok(Json(page))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Public,
    Confidential,
}

#[derive(Debug, Deserialize)]
pub struct CreateClientPayload {
    pub client_id: String,
    pub kind: ClientKind,
    pub client_secret: Option<String>,
    pub allowed_scopes: Scope,
    pub allowed_redirect_uris: HashSet<url::Url>,
}

#[post("/api/v1beta1/clients")]
pub async fn create_client(
    storage: Data<CouchStorage>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    body: Json<CreateClientPayload>,
) -> ApiResult<Json<ClientResponse>> {
    Scope::from("clients:manage").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("clients"), "create")?;

    let client_id = body.client_id.clone();
    let client_secret = body.client_secret.clone();
    let allowed_scopes = body.allowed_scopes.clone();
    let allowed_redirect_uris = body.allowed_redirect_uris.clone();
    let kind = &body.kind;

    log::debug!("creating new {:?} client '{}'", kind, &client_id);

    let client = match kind {
        ClientKind::Public => Client::public(client_id, allowed_scopes, allowed_redirect_uris),
        ClientKind::Confidential => {
            let client_secret = match client_secret {
                Some(client_secret) => client_secret,
                None => {
                    return Err(ApiError::ValidationError(vec![
                        "client_secret is required for confidential clients".to_string(),
                    ]))
                }
            };

            Client::confidential(
                client_id,
                client_secret,
                allowed_scopes,
                allowed_redirect_uris,
            )?
        }
    };

    log::debug!("saving client");
    let client = storage.save_client(client).await?;
    log::debug!("client saved");
    Ok(Json(ClientResponse::from(client)))
}

#[derive(Debug, Deserialize)]
pub struct ClientPathParam {
    pub client_id: String,
}

#[get("/api/v1beta1/clients/{client_id}")]
pub async fn get_client(
    storage: Data<CouchStorage>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<ClientPathParam>,
) -> ApiResult<Json<ClientResponse>> {
    Scope::from("clients:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let client_id = &path.client_id;
    enforcer.check(
        current_user.id(),
        &ClientEntity::build_guid(client_id),
        "read",
    )?;

    let client = storage.get_client(client_id).await;

    client
        .ok_or_else(|| ApiError::not_found(&format!("client '{}' not found", client_id)))
        .map(ClientResponse::from)
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub struct UpdateClientPayload {
    pub client_secret: Option<String>,
    pub allowed_scopes: Option<Scope>,
    pub allowed_redirect_uris: Option<HashSet<url::Url>>,
}

#[put("/api/v1beta1/clients/{client_id}")]
pub async fn update_client(
    storage: Data<CouchStorage>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<ClientPathParam>,
    body: Json<UpdateClientPayload>,
) -> ApiResult<Json<ClientResponse>> {
    Scope::from("clients:manage").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("clients"), "update")?;

    let client_id = &path.client_id;

    log::debug!("updating client '{}'", client_id);

    let mut client = storage
        .get_client(client_id)
        .await
        .ok_or_else(|| ApiError::not_found(&format!("client '{}' not found", client_id)))?;

    if let Some(client_secret) = &body.client_secret {
        client.set_client_secret(client_secret.clone())?;
    }

    if let Some(allowed_scopes) = &body.allowed_scopes {
        client.set_allowed_scopes(allowed_scopes.clone());
    }

    if let Some(allowed_redirect_uris) = &body.allowed_redirect_uris {
        client.set_allowed_redirect_uris(allowed_redirect_uris.clone());
    }

    log::debug!("saving client");
    let client = storage.save_client(client).await?;
    log::debug!("client saved");
    Ok(Json(ClientResponse::from(client)))
}

#[delete("/api/v1beta1/clients/{client_id}")]
pub async fn delete_client(
    storage: Data<CouchStorage>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<ClientPathParam>,
) -> ApiResult<Json<ClientResponse>> {
    Scope::from("clients:manage").matches(&scope)?;
    let enforcer = enforcer.read().await;
    let client_id = &path.client_id;
    enforcer.check(
        current_user.id(),
        &ClientEntity::build_guid(client_id),
        "delete",
    )?;

    let client = storage
        .get_client(client_id)
        .await
        .ok_or_else(|| ApiError::not_found(&format!("client '{}' not found", client_id)))?;

    log::debug!("deleting client");
    storage.delete_client(&client).await?;
    log::debug!("client deleted");

    Ok(Json(ClientResponse::from(client)))
}
