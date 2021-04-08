use url::Url;
use couchdb::Couch;
use crate::config::Configuration;
use api::Client;

mod config;
mod controllers;
mod error;
mod grpc;
mod http;
mod logger;
mod resources;
mod tls;

type ServerResult = Result<(), crate::error::Error>;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = &Configuration::new()?;
    let logger = logger::create_logger(cfg);
    slog::info!(logger, "Config: {:?}", cfg);

    let couch = Couch::new(Url::parse("http://localhost:5984").unwrap(), "enseada".to_string(), "enseada".to_string());
    let client = Client::new(format!("[::1]:{}", cfg.grpc().port()), None)?;
    let (_grpc_ready_tx, grpc_ready_rx) = grpc::ready_channel();

    slog::info!(logger, "Starting API server");
    tokio::try_join!(
        tokio::spawn(http::start(logger.new(slog::o!("server" => "http")))),
        tokio::spawn(grpc::start(cfg, logger.new(slog::o!("server" => "grpc")), &couch)),
        tokio::spawn(controllers::users::start(logger.new(slog::o!("controller" => "users")), &client, grpc_ready_rx.clone()))
    )?;

    Ok(())
}
