use controller_runtime::{ControllerError, ArbiterHandle, start_controller};
use couchdb::Couch;
use slog::Logger;
use crate::controller::v1alpha1::UserController;
use std::time::Duration;

pub mod controller;

pub async fn start(logger: Logger, couch: Couch, arbiter: &ArbiterHandle, polling_interval: Duration) -> Result<(), ControllerError> {
    start_controller(logger, couch, arbiter, polling_interval, UserController::new).await
}
