use crate::controller::v1alpha1::UserController;
use controller_runtime::{start_controller, ArbiterHandle, ControllerError};
use couchdb::Couch;
use slog::Logger;
use std::time::Duration;

pub mod api;
pub mod controller;

pub async fn start(
    logger: Logger,
    couch: Couch,
    arbiter: &ArbiterHandle,
    polling_interval: Duration,
) -> Result<(), ControllerError> {
    start_controller(
        logger,
        couch,
        arbiter,
        polling_interval,
        UserController::new,
    )
    .await
}
