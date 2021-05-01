use std::sync::Arc;

use slog::Logger;
use tokio::sync::RwLock;

use api::{GroupVersion, Resource};
use controller_runtime::{ArbiterHandle, ControllerError, Couch, ResourceManager, start_controller, Reconciler};
pub use policy::*;
pub use policy_attachment::*;
pub use role_attachment::*;

use crate::enforcer::Enforcer;

use super::API_GROUP;
use crate::EnforcerReloader;

mod policy;
mod policy_attachment;
mod role_attachment;

lazy_static! {
    pub static ref API_VERSION: GroupVersion = GroupVersion {
        group: API_GROUP.clone(),
        version: "v1alpha1".to_string(),
    };
}

pub fn create_enforcer_and_reloader(logger: Logger, couch: Couch) -> (Arc<RwLock<Enforcer>>, EnforcerReloader) {
    let db = couch.database(&API_VERSION.group, true);

    let policy_kind = Policy::type_meta().kind_plural;
    let policy_manager = ResourceManager::new(logger.new(slog::o!("manager" => policy_kind)), db.clone());

    let policy_attachment_kind = PolicyAttachment::type_meta().kind_plural;
    let policy_attachment_manager = ResourceManager::new(logger.new(slog::o!("manager" => policy_attachment_kind)), db.clone());

    let role_attachment_kind = RoleAttachment::type_meta().kind_plural;
    let role_attachment_manager = ResourceManager::new(logger.new(slog::o!("manager" => role_attachment_kind)), db.clone());

    let enforcer = Arc::new(RwLock::new(Enforcer::new()));
    let reloader = EnforcerReloader::new(enforcer.clone(), policy_manager, policy_attachment_manager, role_attachment_manager);
    (enforcer, reloader)
}

pub async fn start_controllers(logger: Logger, couch: Couch, controller_arbiter: &ArbiterHandle, polling_interval: std::time::Duration, enforcer_reloader: EnforcerReloader) -> Result<(), ControllerError> {
    let policy_reloader = enforcer_reloader.clone();
    let policy_attachment_reloader = enforcer_reloader.clone();
    let role_attachment_reloader = enforcer_reloader.clone();
    tokio::try_join!(
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, PolicyController::new),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, PolicyAttachmentController::new),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, RoleAttachmentController::new),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, move |_logger, _manager: ResourceManager<Policy>| policy_reloader.clone()),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, move |_logger, _manager: ResourceManager<PolicyAttachment>| policy_attachment_reloader.clone()),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, move |_logger, _manager: ResourceManager<RoleAttachment>| role_attachment_reloader.clone()),
    ).map(|_| ())

}
