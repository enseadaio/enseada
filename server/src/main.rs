use anyhow::Context;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Router;
use axum::routing::get;
use clap::Parser;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use tracing::info;


use futon::Couch;
use futon::model::Up;

use crate::config::Config;

mod api;
mod config;
mod logger;

async fn liveness_check() -> StatusCode {
    StatusCode::OK
}

async fn readiness_check(Extension(couch): Extension<Couch>) -> StatusCode {
    match couch.up().await.unwrap() {
        Up::Ok => StatusCode::OK,
        Up::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    }
}

fn routes(couch: Couch) -> Router {
    Router::new()
        .route("/health/live", get(liveness_check))
        .route("/health/ready", get(readiness_check))
        .nest("/apis", api::routes())
        .layer(ServiceBuilder::new()
            .layer(AddExtensionLayer::new(couch)))
}

async fn run() -> anyhow::Result<()> {
    let cfg: Config = Config::parse();

    logger::try_init(&cfg)?;

    let couch = Couch::new(cfg.couchdb_url.clone());
    let addr = cfg.http_address();

    let app = routes(couch);

    info!("Enseada HTTP server started on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("failed to start HTTP server")
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let _ = dotenv::dotenv();

    if let Err(err) = run().await {
        eprintln!("{}", err)
    }
}

#[cfg(unix)]
pub async fn shutdown_signal() {
    use std::io;
    use tokio::signal::unix::SignalKind;

    async fn terminate() -> io::Result<()> {
        tokio::signal::unix::signal(SignalKind::terminate())?
            .recv()
            .await;
        Ok(())
    }

    tokio::select! {
        _ = terminate() => {},
        _ = tokio::signal::ctrl_c() => {},
    }
    info!("signal received, starting graceful shutdown")
}

#[cfg(windows)]
pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("faild to install CTRL+C handler");
    info!("signal received, starting graceful shutdown")
}
