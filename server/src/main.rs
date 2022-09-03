use anyhow::Context as _;
use axum::routing::get;
use axum::Router;
use clap::Parser;

use futures::stream::StreamExt;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use futon::Couch;

use crate::config::Config;

type Result<T> = anyhow::Result<T>;

mod api;
mod config;
mod logger;

mod health {
    use anyhow::Context;
    use axum::http::StatusCode;
    use axum::{headers, Extension, TypedHeader};

    use futon::Couch;
    use libapi::ApiResult;

    pub async fn live(
        TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
        Extension(couch): Extension<Couch>,
    ) -> ApiResult<StatusCode> {
        let up = couch.up().await.context("error fetching CouchDB status")?;
        tracing::trace!(?up, %user_agent, "liveness check");
        Ok(up.into())
    }

    pub async fn ready(TypedHeader(user_agent): TypedHeader<headers::UserAgent>) -> StatusCode {
        tracing::trace!(%user_agent,"readiness check");
        StatusCode::OK
    }
}

fn routes(couch: &Couch) -> Router {
    Router::new()
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .nest("/apis", api::routes(couch))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(AddExtensionLayer::new(couch.clone())),
        )
}

async fn server(cfg: &Config, couch: &Couch) -> anyhow::Result<()> {
    let addr = cfg.http_address();

    let app = routes(couch);

    info!("Enseada HTTP server started on http://{}", addr);
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("failed to start HTTP server")
}

async fn run() -> anyhow::Result<()> {
    let cfg: Config = Config::parse();
    let id = cfg.node_id();

    logger::try_init(&cfg)?;

    let couch = Couch::new(cfg.couchdb_url.clone());

    init_modules(&couch).await?;
    info!("Modules initialization completed");

    tokio::select! {
        res = auth::start(&id, &couch) => res.context("auth module exited with error"),
        res = server(&cfg, &couch) => res,
    }
}

async fn init_modules(couch: &Couch) -> Result<()> {
    auth::try_init(couch)
        .await
        .context("failed to initialize auth module")?;

    Ok(())
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let _ = dotenv::dotenv();

    if let Err(error) = run().await {
        if tracing::enabled!(tracing::Level::ERROR) {
            tracing::error!("{error:#}");
        } else {
            eprintln!("{error:?}");
        }

        std::process::exit(1)
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
