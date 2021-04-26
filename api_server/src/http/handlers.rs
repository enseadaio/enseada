use std::convert::Infallible;
use std::error::Error as StdError;

use futures::StreamExt;
use hyper::StatusCode;
use slog::Logger;
use warp::{Rejection, Reply, reply};
use warp::body::BodyDeserializeError;
use warp::reject::MethodNotAllowed;
use warp::reply::{json, Json, with_status};
use warp::sse::Event;

use api::core::v1alpha1::List;
use api::error::{Code, ErrorResponse};
use api::Resource;
use controller_runtime::{Arbiter, ResourceManager, Watcher};
use couchdb::Couch;

use crate::error::Error;
use std::time::Duration;

pub(super) async fn watch_resources<T: 'static + Resource + Unpin>(logger: Logger, couch: Couch) -> Result<impl warp::Reply, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    let stream = Watcher::<T>::start(logger.clone(), manager, &Arbiter::current(), Duration::from_secs(60 * 5), Some("now".to_string()));
    let stream = stream.map(|res| match res {
        Ok(event) => Ok::<Event, Infallible>(Event::default().event("change").json_data(event).unwrap()),
        Err(err) => Ok(Event::default().event("error").json_data::<ErrorResponse>(Error::from(err).into()).unwrap()),
    });
    Ok(warp::sse::reply(stream))
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

    eprintln!("{:?}", err);
    if err.is_not_found() || err.find::<MethodNotAllowed>().is_some() {
        code = Code::NotFound;
        message = "not found".to_string();
    } else if let Some(Error::ApiError { code: err_code, message: err_message }) = err.find::<Error>() {
        code = *err_code;
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

