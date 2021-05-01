use std::sync::Arc;

use tokio::sync::RwLock;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager};

use crate::api::v1alpha1::{Policy, PolicyAttachment, RoleAttachment};
use crate::Enforcer;

#[derive(Clone)]
pub struct EnforcerReloader {
    enforcer: Arc<RwLock<Enforcer>>,
    policy_manager: ResourceManager<Policy>,
    policy_attachment_manager: ResourceManager<PolicyAttachment>,
    role_attachment_manager: ResourceManager<RoleAttachment>,
}

impl EnforcerReloader {
    pub fn new(enforcer: Arc<RwLock<Enforcer>>, policy_manager: ResourceManager<Policy>, policy_attachment_manager: ResourceManager<PolicyAttachment>, role_attachment_manager: ResourceManager<RoleAttachment>) -> Self {
        Self {
            enforcer,
            policy_manager,
            policy_attachment_manager,
            role_attachment_manager,
        }
    }

    pub async fn trigger_reload(&self) -> Result<(), ControllerError> {
        let policies = self.policy_manager.list_all().await?;
        let policy_attachments = self.policy_attachment_manager.list_all().await?;
        let role_attachments = self.role_attachment_manager.list_all().await?;

        let mut enforcer = self.enforcer.write().await;
        enforcer.load_rules(policies, policy_attachments, role_attachments);
        Ok(())
    }
}

#[async_trait]
impl Reconciler<Policy> for EnforcerReloader {
    async fn reconcile(&mut self, resource: Policy) -> Result<(), ReconciliationError<ControllerError>> {
        // The resource controller will probably do something on first creation. Let's wait for the next event to reload the engine.
        if !resource.metadata.is_just_created() {
            self.trigger_reload().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Reconciler<PolicyAttachment> for EnforcerReloader {
    async fn reconcile(&mut self, resource: PolicyAttachment) -> Result<(), ReconciliationError<ControllerError>> {
        // The resource controller will probably do something on first creation. Let's wait for the next event to reload the engine.
        if !resource.metadata.is_just_created() {
            self.trigger_reload().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Reconciler<RoleAttachment> for EnforcerReloader {
    async fn reconcile(&mut self, resource: RoleAttachment) -> Result<(), ReconciliationError<ControllerError>> {
        // The resource controller will probably do something on first creation. Let's wait for the next event to reload the engine.
        if !resource.metadata.is_just_created() {
            self.trigger_reload().await?;
        }
        Ok(())
    }
}
