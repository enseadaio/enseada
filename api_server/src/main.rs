#[macro_use]
extern crate lazy_static;

use actix::Arbiter;
use futures::TryFutureExt;

use controller_runtime::start_controller;
use couchdb::Couch;

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

    let enforcer = acl::api::v1alpha1::create_enforcer(logger.clone(), couch.clone());

    slog::info!(logger, "Starting API server");
    tokio::try_join!(
        http::start(logger.new(slog::o!("server" => "http")), couch.clone(), cfg, enforcer.clone()),
        start_controller(logger.clone(), couch.clone(), &controller_arbiter, cfg.controllers().users().polling_interval(), users::api::v1alpha1::UserController::new).map_err(Error::from),
        acl::api::v1alpha1::start_controllers(logger.clone(), couch.clone(), &controller_arbiter, cfg.controllers().users().polling_interval(), enforcer).map_err(Error::from),
    )?;

    Ok(())
}
