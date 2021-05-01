use slog::Logger;

use api::GroupVersion;
pub use client::*;
use controller_runtime::{ArbiterHandle, ControllerError, Couch, start_controller};
pub use user::*;

use super::API_GROUP;

mod client;
mod user;

lazy_static! {
    pub static ref API_VERSION: GroupVersion = GroupVersion {
        group: API_GROUP.clone(),
        version: "v1alpha1".to_string(),
    };
}

pub async fn start_controllers(logger: Logger, couch: Couch, controller_arbiter: &ArbiterHandle, polling_interval: std::time::Duration) -> Result<(), ControllerError> {
    tokio::try_join!(
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, OAuthClientController::new),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, UserController::new),
    ).map(|_| ())
}

