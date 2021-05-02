use std::convert::Infallible;
use std::sync::Arc;

use http::{StatusCode, HeaderMap, HeaderValue};
use serde::Serialize;
use slog::Logger;
use tokio::sync::RwLock;
use url::Url;
use warp::{Filter, Rejection, Reply, reject};
use warp::reply::{json, with_status, Json};

use acl::Enforcer;
use api::{GroupKindName, KindNamedRef};
use auth::handler::OAuthHandler;
use controller_runtime::ResourceManager;
use couchdb::Couch;

use crate::config::Configuration;
use crate::http::{with_enforcer, with_manager};
use warp::path::FullPath;
use futures::Stream;

pub fn mount_can_i(enforcer: Arc<RwLock<Enforcer>>) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    warp::post()
        .and(warp::path("can-i"))
        .and(warp::path::end())
        .and(with_enforcer(enforcer))
        .and(warp::body::json())
        .and_then(can_i)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanIQuery {
    user: String,
    resource: GroupKindName,
    action: String,
}

pub async fn can_i(enforcer: Arc<RwLock<Enforcer>>, query: CanIQuery) -> Result<impl Reply, Infallible> {
    // TODO extract real user from token
    let user = &KindNamedRef {
        kind: "User".to_string(),
        name: query.user.clone(),
    };
    let obj = &query.resource;
    let act = &query.action;

    let enforcer = enforcer.read().await;
    match enforcer.check(user, obj, act) {
        Ok(()) => {
            let json = json(&serde_json::json!({
                "access": "granted",
            }));
            Ok(with_status(json, StatusCode::OK))
        }
        Err(err) => {
            let json = json(&serde_json::json!({
                "access": "denied",
                "reason": err.to_string()
            }));
            Ok(with_status(json, StatusCode::FORBIDDEN))
        }
    }

}

pub fn oauth_routes(logger: Logger, couch: &Couch, cfg: &Configuration) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    let with_handler = with_oauth_handler(logger.clone(), couch,cfg);
    let prefix = warp::path("oauth");
    let authorize = prefix.and(warp::path("authorize")
        .and(warp::get()
            .and(with_handler.clone())
            .and_then(handle_authorization_get_request)
            .or(warp::post().and(with_handler.clone()).and_then(handle_authorization_post_request))
        )
    );
    let token = prefix.and(warp::path("token")).and(warp::post()).and(with_handler.clone()).and_then(handle_token_request);
    let introspect = prefix.and(warp::path("introspect")).and(warp::post()).and(with_handler.clone()).and_then(handle_introspection_request);
    let revoke = prefix.and(warp::path("revoke")).and(warp::post()).and(with_handler.clone()).and_then(handle_revocation_request);
    let logout = prefix.and(warp::path("logout")).and(warp::get()).and(with_handler.clone()).and_then(handle_logout_request);
    let userinfo = prefix.and(warp::path("userinfo")).and(warp::get()).and(with_handler).and_then(handle_userinfo_request);
    let metadata = warp::path(".well-known").and(warp::path("oauth-authorization-server")).and(warp::get()).and(with_public_host(cfg)).and_then(handle_oauth_metadata);

    metadata
        .or(authorize)
        .or(token)
        .or(introspect)
        .or(revoke)
        .or(logout)
        .or(userinfo)
}

fn with_oauth_handler(logger: Logger, couch: &Couch, cfg: &Configuration) -> impl Filter<Extract=(OAuthHandler,), Error=Infallible> + Clone {
    let db = couch.database(&auth::api::API_GROUP, true);
    let client_manager = ResourceManager::new(logger.clone(), db.clone());
    let auth_code_manager = ResourceManager::new(logger.clone(), db);
    let handler = OAuthHandler::new(client_manager, auth_code_manager, cfg.secret_key_base().to_string());
    warp::any().map(move || handler.clone())
}

async fn handle_authorization_get_request(handler: OAuthHandler) -> Result<impl Reply, Rejection> {
    Ok("GET /oauth/authorize")
}

async fn handle_authorization_post_request(handler: OAuthHandler) -> Result<impl Reply, Rejection> {
    Ok("POST /oauth/authorize")
}

async fn handle_token_request(handler: OAuthHandler) -> Result<impl Reply, Rejection> {
    Ok("POST /oauth/token")
}

async fn handle_introspection_request(handler: OAuthHandler) -> Result<impl Reply, Rejection> {
    Ok("POST /oauth/introspect")
}

async fn handle_revocation_request(handler: OAuthHandler) -> Result<impl Reply, Rejection> {
    Ok("POST /oauth/revoke")
}

async fn handle_logout_request(handler: OAuthHandler) -> Result<impl Reply, Rejection> {
    Ok("GET /oauth/logout")
}

async fn handle_userinfo_request(handler: OAuthHandler) -> Result<impl Reply, Rejection> {
    Ok("GET /oauth/userinfo")
}

fn with_public_host(cfg: &Configuration) -> impl Filter<Extract=(Url,), Error=Infallible> + Clone {
    let host = cfg.public_host().clone();
    warp::any().map(move || host.clone())
}

#[derive(Clone, Debug, Serialize)]
pub struct OAuthMetadata {
    issuer: String,
    authorization_endpoint: Url,
    token_endpoint: Url,
    revocation_endpoint: Url,
    introspection_endpoint: Url,
    end_session_endpoint: Url,
    userinfo_endpoint: Url,
    grant_types_supported: Vec<String>,
    response_types_supported: Vec<String>,
    response_modes_supported: Vec<String>,
    token_endpoint_auth_methods_supported: Vec<String>,
    revocation_endpoint_auth_methods_supported: Vec<String>,
    introspection_endpoint_auth_methods_supported: Vec<String>,
    code_challenge_methods_supported: Vec<String>,
    service_documentation: Url,
}

async fn handle_oauth_metadata(host: Url) -> Result<Json, Rejection> {
    Ok(json(&OAuthMetadata {
        issuer: host.to_string(),
        authorization_endpoint: host.join("/oauth/authorize").unwrap(),
        token_endpoint: host.join("/oauth/token").unwrap(),
        revocation_endpoint: host.join("/oauth/revoke").unwrap(),
        introspection_endpoint: host.join("/oauth/introspect").unwrap(),
        end_session_endpoint: host.join("/oauth/logout").unwrap(),
        userinfo_endpoint: host.join("/oauth/me").unwrap(),
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ],
        response_types_supported: vec!["code".to_string()],
        response_modes_supported: vec!["query".to_string()],
        code_challenge_methods_supported: vec!["plain".to_string(), "S256".to_string()],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "client_secret_post".to_string(),
        ],
        revocation_endpoint_auth_methods_supported: vec!["client_secret_basic".to_string()],
        introspection_endpoint_auth_methods_supported: vec!["client_secret_basic".to_string()],
        service_documentation: Url::parse("https://docs.enseada.io").unwrap(),
    }))
}
