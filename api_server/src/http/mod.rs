use std::convert::Infallible;
use std::net::SocketAddr;

use slog::Logger;
use warp::Filter;

use couchdb::Couch;
use handlers::*;

use crate::ServerResult;
use crate::error::handle_rejection;

mod handlers;

pub async fn start(logger: Logger, couch: Couch) -> ServerResult {
    let addr: SocketAddr = "[::]:9623".parse().unwrap();

    let watch_list_path = warp::path!("apis" / String / String / String / "watch");
    let resource_list_path = warp::path!("apis" / String / String / String);
    let resource_path = warp::path!("apis" / String / String / String / String);

    let req_logger = logger.clone();
    let routes = warp::path!("healthz").and_then(health)
        .or(warp::path!("readyz").and_then(health))
        .or(warp::get()
            .and(with_logger(logger.new(slog::o!("handler" => "watch"))))
            .and(with_couch(couch.clone()))
            .and(watch_list_path).and_then(watch_resources))
        .or(warp::get()
            .and(with_logger(logger.new(slog::o!("handler" => "list"))))
            .and(with_couch(couch.clone()))
            .and(resource_list_path).and_then(list_resources))
        .or(warp::get()
            .and(with_logger(logger.new(slog::o!("handler" => "get"))))
            .and(with_couch(couch.clone())).and(resource_path).and_then(get_resource))
        .or(warp::put()
            .and(with_logger(logger.new(slog::o!("handler" => "create"))))
            .and(with_couch(couch.clone()))
            .and(resource_path).and(warp::body::json::<serde_json::Value>()).and_then(create_resource))
        .or(warp::patch()
            .and(with_logger(logger.new(slog::o!("handler" => "update"))))
            .and(with_couch(couch.clone()))
            .and(resource_path).and(warp::body::json::<serde_json::Value>()).and_then(update_resource))
        .or(warp::delete()
            .and(with_logger(logger.new(slog::o!("handler" => "delete"))))
            .and(with_couch(couch.clone()))
            .and(resource_path).and_then(delete_resource))
        .with(warp::cors().allow_any_origin())
        .with(warp::log::custom(move |info| {
            slog::info!(req_logger, ""; slog::o!(
                "method" => info.method().to_string(),
                "path" => info.path().to_string(),
                "status" => info.status().as_u16(),
            ));
        }))
        .recover(handle_rejection);


    slog::info!(logger, "HTTP server listening on http://{}", &addr);
    warp::serve(routes)
        .run(addr)
        .await;

    Ok(())
}

fn with_logger(logger: Logger) -> impl Filter<Extract=(Logger,), Error=Infallible> + Clone {
    warp::any().map(move || logger.clone())
}

fn with_couch(couch: Couch) -> impl Filter<Extract=(Couch,), Error=Infallible> + Clone {
    warp::any().map(move || couch.clone())
}
