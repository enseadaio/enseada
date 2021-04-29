use std::sync::Arc;

use slog::Logger;
use tokio::sync::RwLock;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use crate::enforcer::Enforcer;

use super::RoleAttachment;
use crate::error::Error;

pub struct RoleAttachmentController {
    logger: Logger,
    manager: ResourceManager<RoleAttachment>,
    enforcer: Arc<RwLock<Enforcer>>,
}

impl RoleAttachmentController {
    pub fn new(
        logger: Logger,
        manager: ResourceManager<RoleAttachment>,
        enforcer: Arc<RwLock<Enforcer>>,
    ) -> Self {
        RoleAttachmentController {
            logger,
            manager,
            enforcer,
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
            // This put will trigger a new event. The Enforcer will be update on the next run
            return Ok(());
        }
        
        let mut enforcer = self.enforcer.write().await;
        if let Err(Error::Controller(err)) = enforcer.load_model_from_resources().await {
            slog::error!(self.logger, "Failed to reload ACL engine: {}", err);
            Err(ReconciliationError::wrap(err))
        } else {
            Ok(())
        }
    }
}
