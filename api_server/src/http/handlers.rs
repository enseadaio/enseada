use warp::Reply;
use std::convert::Infallible;
use warp::http::response::Builder;
use futures::stream;

pub(super) async fn health() -> Result<impl Reply, Infallible> {
    Ok("UP".to_string())
}

pub(super) async fn watch_resources(group: String, version: String, kind: String) -> Result<impl warp::Reply, Infallible> {
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

pub(super) async fn watch_resource(group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Watching resource {}/{} {} {}", group, version, kind, name))
}

pub(super) async fn list_resources(group: String, version: String, kind: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Listing resources {}/{} {}", group, version, kind))
}

pub(super) async fn get_resource(group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Getting resource {}/{} {} {}", group, version, kind, name))
}

pub(super) async fn create_resource(group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Creating resource {}/{} {} {}", group, version, kind, name))
}

pub(super) async fn update_resource(group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Updating resource {}/{} {} {}", group, version, kind, name))
}

pub(super) async fn delete_resource(group: String, version: String, kind: String, name: String) -> Result<impl Reply, Infallible> {
    Ok(format!("Deleting resource {}/{} {} {}", group, version, kind, name))
}
