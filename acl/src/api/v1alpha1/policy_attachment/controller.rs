use slog::Logger;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use super::PolicyAttachment;

pub struct PolicyAttachmentController {
    logger: Logger,
    manager: ResourceManager<PolicyAttachment>,
}

impl PolicyAttachmentController {
    pub fn new(
        logger: Logger,
        manager: ResourceManager<PolicyAttachment>,
    ) -> Self {
        PolicyAttachmentController {
            logger,
            manager,
        }
    }
}

#[async_trait]
impl Reconciler<PolicyAttachment> for PolicyAttachmentController {
    async fn reconcile(
        &mut self,
        mut policy_attachment: PolicyAttachment,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        if policy_attachment.metadata.is_just_created() {
            policy_attachment.metadata.created_at = Some(Utc::now());
            let name = policy_attachment.metadata.name.clone();
            self.manager.put(&name, policy_attachment).await?;
        }
        Ok(())
    }
}
