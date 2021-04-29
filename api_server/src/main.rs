#[macro_use]
extern crate lazy_static;

use actix::Arbiter;
use futures::TryFutureExt;

use controller_runtime::start_controller;
use couchdb::Couch;
use users::controller::v1alpha1;

use crate::config::Configuration;
use crate::error::Error;

mod config;
mod error;
mod http;
mod logger;
mod telemetry;

type ServerResult = Result<(), Error>;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = &Configuration::new()?;
    let logger = logger::create_logger(cfg);
    slog::debug!(logger, ""; "config" => cfg.pretty_print());

    let couch_cfg = cfg.couchdb();
    let couch = &Couch::new(
        couch_cfg.url().clone(),
        couch_cfg.username().to_string(),
        couch_cfg.password().to_string(),
    );

    let controller_arbiter = Arbiter::new().handle();

    slog::info!(logger, "Starting API server");
    tokio::try_join!(
        http::start(logger.new(slog::o!("server" => "http")), couch.clone(), cfg),
        start_controller(
            logger.clone(),
            couch.clone(),
            &controller_arbiter,
            cfg.controllers().users().polling_interval(),
            v1alpha1::UserController::new
        )
        .map_err(Error::from),
    )?;

    Ok(())
}
