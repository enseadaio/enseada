use std::convert::Infallible;
use std::error::Error as StdError;

use futures::stream;
use hyper::StatusCode;
use slog::Logger;
use warp::{Rejection, Reply, reply};
use warp::body::BodyDeserializeError;
use warp::http::response::Builder;
use warp::reply::{json, Json, with_status};

use api::core::v1alpha1::List;
use api::error::{Code, ErrorResponse};
use api::Resource;
use controller_runtime::ResourceManager;
use couchdb::Couch;

use crate::error::Error;

pub(super) async fn health() -> Result<impl Reply, Rejection> {
    Ok("UP".to_string())
}

pub(super) async fn watch_resources(logger: Logger, couch: Couch) -> Result<impl warp::Reply, Rejection> {
    let vec: Vec<Result<String, Infallible>> = vec![
        Ok("todo".to_string())
    ];
    let body = hyper::Body::wrap_stream(stream::iter(vec));
    Ok(Builder::default()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap())
}

pub(super) async fn watch_resource<T: Resource>(logger: Logger, couch: Couch, name: String) -> Result<impl Reply, Rejection> {
    Ok("Watching resource")
}

pub(super) async fn list_resources<T: Resource>(logger: Logger, couch: Couch) -> Result<Json, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    let items = manager.list().await.map_err(Error::from)?;
    let list = List::from(items);
    Ok(json(&list))
}

pub(super) async fn get_resource<T: Resource>(logger: Logger, couch: Couch, name: String) -> Result<Json, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    let resource = manager.get(&name).await.map_err(Error::from)?;
    Ok(json(&resource))
}

pub(super) async fn create_resource<T: Resource>(logger: Logger, couch: Couch, name: String, body: T) -> Result<Json, Rejection> {
    let manager = create_manager(logger.clone(), couch).await?;
    let body = normalize_resource(&manager, &name, body).await?;
    let resource = manager.put(&name, body).await.map_err(Error::from)?;
    Ok(json(&resource))
}

pub(super) async fn update_resource<T: Resource>(logger: Logger, couch: Couch, name: String, body: T) -> Result<Json, Rejection> {
    let manager = create_manager(logger.clone(), couch).await?;
    let body = normalize_resource(&manager, &name, body).await?;
    let resource = manager.put(&name, body).await.map_err(Error::from)?;
    Ok(json(&resource))
}

pub(super) async fn delete_resource<T: Resource>(logger: Logger, couch: Couch, name: String) -> Result<impl Reply, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    manager.delete(&name).await.map_err(Error::from)?;
    Ok(with_status(reply::reply(), StatusCode::NO_CONTENT))
}

async fn create_manager<T: Resource>(logger: Logger, couch: Couch) -> Result<ResourceManager<T>, Error> {
    let typ = T::type_meta();
    let group = &typ.api_version.group;
    let version = &typ.api_version.version;
    let kind = &typ.kind_plural;

    let db = couch.database(&group, true);
    if !db.exists_self().await.map_err(Error::from)? {
        return Err(Error::not_found(format!("the server doesn't have a resource type '{}' in group {}/{}", kind, group, version)));
    }
    Ok(ResourceManager::new(logger, db, kind))
}

async fn normalize_resource<T: Resource>(manager: &ResourceManager<T>, name: &str, mut body: T) -> Result<T, Error> {
    let existing = manager.find(name).await?;

    if let Some(existing) = existing {
        body.set_status(existing.status().cloned());
        body.set_metadata(existing.metadata().clone());
    } else {
        body.set_status(None);
    }

    Ok(body)
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;
    let metadata = None;

    if err.is_not_found() {
        code = Code::NotFound;
        message = "not found".to_string();
    } else if let Some(Error::ApiError { code: err_code, message: err_message }) = err.find::<Error>() {
        code = err_code.clone();
        message = err_message.clone();
    } else if let Some(err) = err.find::<BodyDeserializeError>() {
        code = Code::InvalidBody;
        message = err.source().map_or_else(|| err.to_string(), |source| source.to_string());
    } else {
        code = Code::Unknown;
        message = "internal server error".to_string();
    }

    let status = code.to_status();
    let json = json(&ErrorResponse {
        code,
        message,
        metadata,
    });

    Ok(with_status(json, status))
}

