use slog::Logger;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use super::RoleAttachment;

pub struct RoleAttachmentController {
    logger: Logger,
    manager: ResourceManager<RoleAttachment>,
}

impl RoleAttachmentController {
    pub fn new(
        logger: Logger,
        manager: ResourceManager<RoleAttachment>,
    ) -> Self {
        RoleAttachmentController {
            logger,
            manager,
        }
    }
}

#[async_trait]
impl Reconciler<RoleAttachment> for RoleAttachmentController {
    async fn reconcile(
        &mut self,
        mut role_attachment: RoleAttachment,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        if role_attachment.metadata.is_just_created() {
            role_attachment.metadata.created_at = Some(Utc::now());
            let name = role_attachment.metadata.name.clone();
            self.manager.put(&name, role_attachment).await?;
        }
        Ok(())
    }
}
