use std::net::SocketAddr;

use slog::Logger;
use warp::Filter;

use handlers::*;

use crate::ServerResult;

mod handlers;

pub async fn start(logger: Logger) -> ServerResult {
    let addr: SocketAddr = "[::]:9623".parse().unwrap();

    let watch_list_path = warp::path!("apis" / String / String / String / "watch");
    let watch_path = warp::path!("apis" / String / String / String / String / "watch");
    let resource_list_path = warp::path!("apis" / String / String / String);
    let resource_path = warp::path!("apis" / String / String / String / String);

    let routes = warp::path!("healthz").and_then(health)
        .or(warp::path!("readyz").and_then(health))
        .or(warp::get().and(watch_list_path)
            .and_then(watch_resources))
        .or(warp::get().and(watch_path).and_then(watch_resource))
        .or(warp::get().and(resource_list_path).and_then(list_resources))
        .or(warp::get().and(resource_path).and_then(get_resource))
        .or(warp::put().and(resource_path).and_then(create_resource))
        .or(warp::patch().and(resource_path).and_then(update_resource))
        .or(warp::delete().and(resource_path).and_then(delete_resource));


    slog::info!(logger, "HTTP server listening on http://{}", &addr);
    warp::serve(routes)
        .run(addr)
        .await;

    Ok(())
}
