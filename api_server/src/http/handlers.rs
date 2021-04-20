use std::convert::Infallible;

use futures::stream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use slog::Logger;
use warp::http::response::Builder;
use warp::{Reply, Rejection, reply};
use warp::reply::{json, Json, with_status};

use api::core::v1alpha1::List;
use couchdb::Couch;

use crate::error::Error;
use crate::resources::ResourceManager;
use hyper::StatusCode;
use serde_json::Value;

pub(super) async fn health() -> Result<impl Reply, Rejection> {
    Ok("UP".to_string())
}

pub(super) async fn watch_resources(logger: Logger, couch: Couch, group: String, version: String, kind: String) -> Result<impl warp::Reply, Rejection> {
    let vec: Vec<Result<String, Infallible>> = vec![
        Ok(format!("{}/{} {}", group, version, kind))
    ];
    let body = hyper::Body::wrap_stream(stream::iter(vec));
    Ok(Builder::default()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap())
}

pub(super) async fn watch_resource(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String) -> Result<impl Reply, Rejection> {
    Ok(format!("Watching resource {}/{} {} {}", group, version, kind, name))
}

pub(super) async fn list_resources(logger: Logger, couch: Couch, group: String, version: String, kind: String) -> Result<Json, Rejection> {
    let manager = create_manager::<Value>(logger.clone(), couch, group, version, kind).await?;
    let items = manager.list().await?;
    let list = List::from(items);
    Ok(json(&list))
}

pub(super) async fn get_resource(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String) -> Result<Json, Rejection> {
    let manager = create_manager::<Value>(logger.clone(), couch, group, version, kind).await?;
    let resource = manager.get(&name).await?;
    Ok(json(&resource))
}

pub(super) async fn create_resource(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String, body: Value) -> Result<Json, Rejection> {
    let manager = create_manager::<Value>(logger.clone(), couch, group.clone(), version.clone(), kind.clone()).await?;
    let body = normalize_resource(&manager, &group, &version, &kind, &name, body).await?;
    let resource =  manager.put(&name, body).await?;
    Ok(json(&resource))
}

pub(super) async fn update_resource(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String, body: Value) -> Result<Json, Rejection> {
    let manager = create_manager::<Value>(logger.clone(), couch, group.to_string(), version.to_string(), kind.to_string()).await?;
    let body = normalize_resource(&manager, &group, &version, &kind, &name, body).await?;
    let resource =  manager.put(&name, body).await?;
    Ok(json(&resource))
}

pub(super) async fn delete_resource(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String) -> Result<impl Reply, Rejection> {
    let manager = create_manager::<Value>(logger.clone(), couch, group, version, kind).await?;
    manager.delete(&name).await?;
    Ok(with_status(reply::reply(), StatusCode::NO_CONTENT))
}

async fn create_manager<T: Clone + DeserializeOwned + Serialize>(logger: Logger, couch: Couch, group: String, version: String, kind: String) -> Result<ResourceManager<T>, Error>{
    let db = couch.database(&group, true);
    if !db.exists_self().await.map_err(Error::from)? {
        return Err(Error::not_found(format!("the server doesn't have a resource type '{}' in group {}/{}", kind, group, version)))
    }
    Ok(ResourceManager::new(logger, db, kind))
}

async fn normalize_resource(manager: &ResourceManager<Value>, group: &str, version: &str, kind: &str, name: &str, mut body: Value) -> Result<Value, Error> {
    let existing_status = manager.find(name).await?.and_then(|res| res.get("status").cloned());
    if let Some(body_object) = body.as_object_mut() {
        if let Some(existing_status) = existing_status {
            body_object.insert("status".to_string(), existing_status);
        }

        body_object.insert("apiVersion".to_string(), format!("{}/{}", group, version).into());
    }

    Ok(body)
}
