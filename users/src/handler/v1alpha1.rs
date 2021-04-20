use std::convert::Infallible;

use slog::Logger;
use warp::Reply;
use warp::reply::{Json, json};

use api::Resource;
use couchdb::Couch;

pub async fn watch(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Watching resource {}/{} {} {}", group, version, kind, name))
}

pub async fn list<T: Resource>(logger: Logger, couch: Couch, group: String, version: String, kind: String) -> Result<Json, Infallible> {
    let db = couch.database(&group, true);
    let manager = ResourceManager::<T>::new(logger.clone(), db, kind);
    let items = manager.list().await.unwrap();
    let list = List::from(items);
    Ok(json(&list))
}

pub async fn get(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Getting resource {}/{} {} {}", group, version, kind, name))
}

pub async fn create<T: Resource>(logger: Logger, couch: Couch, body: T) -> Result<Json, Infallible> {
    let db = couch.database("users-v1alpha1", true);
    let manager = ResourceManager::new(logger.clone(), db, "user");
    let user =  manager.put(&name, body).await.unwrap();
    Ok(json(&user))
}

pub async fn update<T: Resource>(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String, body: T) -> Result<impl Reply, Infallible> {
    Ok(format!("Updating resource {}/{} {} {}", group, version, kind, name))
}

pub async fn delete(logger: Logger, couch: Couch, group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Deleting resource {}/{} {} {}", group, version, kind, name))
}
