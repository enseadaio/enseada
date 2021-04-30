use std::convert::Infallible;
use std::sync::Arc;

use http::Method;
use slog::Logger;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply};

use acl::Enforcer;
use api::Resource;
use couchdb::Couch;
use handlers::*;

use crate::config::Configuration;
use crate::config::tls::Tls;
use crate::ServerResult;

mod handlers;
mod telemetry;

pub async fn start(logger: Logger, couch: Couch, cfg: &Configuration, enforcer: Arc<RwLock<Enforcer>>) -> ServerResult {
    let addr = cfg.http().address();

    let routes = telemetry::routes()
        .or(warp::path("apis")
            .and(can_i(enforcer.clone())
                .or(mount_resource::<users::api::v1alpha1::User>(logger.new(slog::o!()), couch.clone(), enforcer.clone()))
                .or(mount_resource::<acl::api::v1alpha1::Policy>(logger.new(slog::o!()), couch.clone(), enforcer.clone()))
                .or(mount_resource::<acl::api::v1alpha1::PolicyAttachment>(logger.new(slog::o!()), couch.clone(), enforcer.clone()))
                .or(mount_resource::<acl::api::v1alpha1::RoleAttachment>(logger.new(slog::o!()), couch.clone(), enforcer.clone()))
            ).recover(handle_rejection))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(vec![
                    "User-Agent",
                    "Sec-Fetch-Mode",
                    "Referer",
                    "Origin",
                    "Access-Control-Request-Method",
                    "Access-Control-Request-Headers",
                    "Content-Type",
                ])
                .allow_methods(vec![
                    Method::OPTIONS,
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::HEAD,
                    Method::TRACE,
                    Method::CONNECT,
                    Method::PATCH,
                ]),
        )
        .with(warp::log::custom(telemetry::log(logger.clone())))
        .with(warp::log::custom(telemetry::http_metrics));

    let tls = cfg.http().tls();
    let protocol = tls.map_or_else(|| "http", |_| "https");
    slog::info!(logger, "HTTP server listening on {}://{}", protocol, &addr);
    let server = warp::serve(routes);
    if let Some(Tls { cert, key }) = tls {
        server.tls().cert_path(cert).key_path(key).run(addr).await;
    } else {
        server.run(addr).await;
    };

    Ok(())
}

fn with_logger(logger: Logger) -> impl Filter<Extract=(Logger, ), Error=Infallible> + Clone {
    warp::any().map(move || logger.clone())
}

fn with_couch(couch: Couch) -> impl Filter<Extract=(Couch, ), Error=Infallible> + Clone {
    warp::any().map(move || couch.clone())
}

fn with_enforcer(enforcer: Arc<RwLock<Enforcer>>) -> impl Filter<Extract=(Arc<RwLock<Enforcer>>, ), Error=Infallible> + Clone {
    warp::any().map(move || enforcer.clone())
}

fn can_i(enforcer: Arc<RwLock<Enforcer>>) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    warp::post()
        .and(warp::path("can-i"))
        .and(warp::path::end())
        .and(with_enforcer(enforcer))
        .and(warp::body::json())
        .and_then(handlers::can_i)
}

fn mount_resource<T: 'static + Resource + Unpin>(
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
