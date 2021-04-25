use std::convert::Infallible;
use std::net::SocketAddr;

use slog::Logger;
use warp::{Filter, Rejection, Reply};

use api::Resource;
use api::users::v1alpha1;
use couchdb::Couch;
use handlers::*;

use crate::ServerResult;

mod handlers;

pub async fn start(logger: Logger, couch: Couch) -> ServerResult {
    let addr: SocketAddr = "[::]:9623".parse().unwrap();

    let watch_list_path = warp::path!("apis" / String / String / String / "watch");
    let resource_list_path = warp::path!("apis" / String / String / String);
    let resource_path = warp::path!("apis" / String / String / String / String);

    let req_logger = logger.clone();
    let routes = warp::path!("healthz").and_then(health)
        .with(warp::cors().allow_any_origin())
        .with(warp::log::custom(move |info| {
            slog::info!(req_logger, ""; slog::o!(
                "method" => info.method().to_string(),
                "path" => info.path().to_string(),
                "status" => info.status().as_u16(),
            ));
        }))
        .or(warp::path!("readyz").and_then(health))
        .or(warp::path("apis")
            .and(mount_resource::<v1alpha1::User>(logger.new(slog::o!()), couch.clone()))
            .recover(handle_rejection));


    slog::info!(logger, "HTTP server listening on http://{}", &addr);
    warp::serve(routes)
        .run(addr)
        .await;

    Ok(())
}

fn with_logger(logger: Logger) -> impl Filter<Extract=(Logger, ), Error=Infallible> + Clone {
    warp::any().map(move || logger.clone())
}

fn with_couch(couch: Couch) -> impl Filter<Extract=(Couch, ), Error=Infallible> + Clone {
    warp::any().map(move || couch.clone())
}

fn mount_resource<T: 'static + Resource>(logger: Logger, couch: Couch) -> impl Filter<Extract=(impl Reply,), Error=Rejection> + Clone {
    let typ = T::type_meta();
    let group = typ.api_version.group;
    let version = typ.api_version.version;
    let kind = typ.kind_plural;

    let mount_point = warp::path(group).and(warp::path(version)).and(warp::path(kind));
    let list_path = mount_point.clone().and(warp::path::end());
    let resource_path = mount_point.clone().and(warp::path::param::<String>());
    let watch_path = mount_point.clone().and(warp::path("watch"));

    let watch = warp::get()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(watch_path)
        .and_then(watch_resources);

    let list = warp::get()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(list_path)
        .and_then(list_resources::<T>);

    let get = warp::get()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(resource_path.clone())
        .and_then(get_resource::<T>);

    let create = warp::put()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(resource_path.clone())
        .and(warp::body::json::<T>())
        .and_then(create_resource);

    let update = warp::patch()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(resource_path.clone())
        .and(warp::body::json::<T>())
        .and_then(update_resource);

    let delete = warp::delete()
        .and(with_logger(logger.clone()))
        .and(with_couch(couch.clone()))
        .and(resource_path)
        .and_then(delete_resource::<T>);

    watch
        .or(list)
        .or(get)
        .or(create)
        .or(update)
        .or(delete)
}
