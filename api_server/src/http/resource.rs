use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use hyper::StatusCode;
use slog::Logger;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply, reply};
use warp::reply::{json, Json, with_status};
use warp::sse::Event;

use acl::Enforcer;
use api::core::v1alpha1::List;
use api::error::ErrorResponse;
use api::Resource;
use controller_runtime::{Arbiter, ResourceManager, Watcher};
use couchdb::Couch;

use crate::error::Error;
use crate::http::{with_couch, with_enforcer, with_logger};

pub(super) async fn watch_resources<T: 'static + Resource + Unpin>(logger: Logger, couch: Couch, enforcer: Arc<RwLock<Enforcer>>) -> Result<impl warp::Reply, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    let stream = Watcher::<T>::start(logger.clone(), manager, &Arbiter::current(), Duration::from_secs(60 * 5), Some("now".to_string()));
    let stream = stream.map(|res| match res {
        Ok(event) => Ok::<Event, Infallible>(Event::default().event("change").json_data(event).unwrap()),
        Err(err) => Ok(Event::default().event("error").json_data::<ErrorResponse>(Error::from(err).into()).unwrap()),
    });
    Ok(warp::sse::reply(stream))
}

pub(super) async fn list_resources<T: Resource>(logger: Logger, couch: Couch, enforcer: Arc<RwLock<Enforcer>>) -> Result<impl Reply, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    let items = manager.list().await.map_err(Error::from)?;
    let list = List::from(items);
    Ok(json(&list))
}

pub(super) async fn get_resource<T: Resource>(logger: Logger, couch: Couch, enforcer: Arc<RwLock<Enforcer>>, name: String) -> Result<impl Reply, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    let resource = manager.get(&name).await.map_err(Error::from)?;
    Ok(json(&resource))
}

pub(super) async fn create_resource<T: Resource>(logger: Logger, couch: Couch, enforcer: Arc<RwLock<Enforcer>>, name: String, body: T) -> Result<impl Reply, Rejection> {
    let manager = create_manager(logger.clone(), couch).await?;
    let body = normalize_resource(&manager, &name, body).await?;
    let resource = manager.put(&name, body).await.map_err(Error::from)?;
    Ok(with_status(json(&resource), StatusCode::ACCEPTED))
}

pub(super) async fn update_resource<T: Resource>(logger: Logger, couch: Couch, enforcer: Arc<RwLock<Enforcer>>, name: String, body: T) -> Result<impl Reply, Rejection> {
    let manager = create_manager(logger.clone(), couch).await?;
    let body = normalize_resource(&manager, &name, body).await?;
    let resource = manager.put(&name, body).await.map_err(Error::from)?;
    Ok(with_status(json(&resource), StatusCode::ACCEPTED))
}

pub(super) async fn delete_resource<T: Resource>(logger: Logger, couch: Couch, enforcer: Arc<RwLock<Enforcer>>, name: String) -> Result<impl Reply, Rejection> {
    let manager = create_manager::<T>(logger.clone(), couch).await?;
    manager.delete(&name).await.map_err(Error::from)?;
    Ok(with_status(reply::reply(), StatusCode::ACCEPTED))
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
    Ok(ResourceManager::new(logger, db))
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

pub fn mount<T: 'static + Resource + Unpin>(
    logger: Logger,
    couch: Couch,
    enforcer: Arc<RwLock<Enforcer>>,
) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    let typ = T::type_meta();
    let group = typ.api_version.group;
    let version = typ.api_version.version;
    let kind = typ.kind_plural;

    let mount_point = warp::path(group)
        .and(warp::path(version))
        .and(warp::path(kind));
    let list_path = mount_point.clone().and(warp::path::end());
    let resource_path = mount_point.clone().and(warp::path::param::<String>());
    let watch_path = mount_point.and(warp::path("watch"));

    let watch = warp::get()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(with_enforcer(enforcer.clone()))
        .and(watch_path)
        .and_then(watch_resources::<T>);

    let list = warp::get()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(with_enforcer(enforcer.clone()))
        .and(list_path)
        .and_then(list_resources::<T>);

    let get = warp::get()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(with_enforcer(enforcer.clone()))
        .and(resource_path.clone())
        .and_then(get_resource::<T>);

    let create = warp::put()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(with_enforcer(enforcer.clone()))
        .and(resource_path.clone())
        .and(warp::body::json::<T>())
        .and_then(create_resource);

    let update = warp::patch()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(with_enforcer(enforcer.clone()))
        .and(resource_path.clone())
        .and(warp::body::json::<T>())
        .and_then(update_resource);

    let delete = warp::delete()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(with_enforcer(enforcer.clone()))
        .and(resource_path)
        .and_then(delete_resource::<T>);

    watch.or(list).or(get).or(create).or(update).or(delete)
}
