use controller_runtime::{ControllerError, ArbiterHandle, start_controller};
use couchdb::Couch;
use slog::Logger;
use crate::controller::v1alpha1::UserController;

pub mod controller;

pub async fn start(logger: Logger, couch: Couch, arbiter: &ArbiterHandle) -> Result<(), ControllerError> {
    start_controller(logger, couch, arbiter, UserController::new).await
}
