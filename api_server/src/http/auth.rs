use std::sync::Arc;
use tokio::sync::RwLock;
use acl::Enforcer;
use warp::{Filter, Reply, Rejection};
use crate::http::with_enforcer;
use api::{GroupVersionKindName, KindNamedRef};
use std::convert::Infallible;
use warp::reply::{with_status, json};
use http::StatusCode;

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
    resource: GroupVersionKindName,
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
