use std::sync::Arc;

use slog::Logger;
use tokio::sync::RwLock;

use api::{GroupVersion, Resource};
use controller_runtime::{ArbiterHandle, ControllerError, Couch, ResourceManager, start_controller};
pub use policy::*;
pub use policy_attachment::*;
pub use role_attachment::*;

use crate::enforcer::Enforcer;

mod policy;
mod policy_attachment;
mod role_attachment;

lazy_static! {
    pub static ref API_VERSION: GroupVersion = GroupVersion {
        group: "acl".to_string(),
        version: "v1alpha1".to_string(),
    };
}

pub fn create_enforcer(logger: Logger, couch: Couch) -> Arc<RwLock<Enforcer>> {
    let db = couch.database(&API_VERSION.group, true);

    let policy_kind = Policy::type_meta().kind_plural;
    let policy_manager = ResourceManager::new(logger.new(slog::o!("manager" => policy_kind)), db.clone());

    let policy_attachment_kind = PolicyAttachment::type_meta().kind_plural;
    let policy_attachment_manager = ResourceManager::new(logger.new(slog::o!("manager" => policy_attachment_kind)), db.clone());

    let role_attachment_kind = RoleAttachment::type_meta().kind_plural;
    let role_attachment_manager = ResourceManager::new(logger.new(slog::o!("manager" => role_attachment_kind)), db.clone());

    let enforcer = Enforcer::new(policy_manager, policy_attachment_manager, role_attachment_manager);
    Arc::new(RwLock::new(enforcer))
}

pub async fn start_controllers(logger: Logger, couch: Couch, controller_arbiter: &ArbiterHandle, polling_interval: std::time::Duration, enforcer: Arc<RwLock<Enforcer>>) -> Result<(), ControllerError> {
    let policy_enforcer = enforcer.clone();
    let policy_attachment_enforcer = enforcer.clone();
    tokio::try_join!(
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, move |logger, manager| PolicyController::new(logger, manager, policy_enforcer.clone())),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, move |logger, manager| PolicyAttachmentController::new(logger, manager, policy_attachment_enforcer.clone())),
        start_controller(logger.clone(), couch.clone(), controller_arbiter, polling_interval, move |logger, manager| RoleAttachmentController::new(logger, manager, enforcer.clone())),
    ).map(|_| ())

}
