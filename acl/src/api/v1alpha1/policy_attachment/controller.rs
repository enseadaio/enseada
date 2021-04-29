use std::sync::Arc;

use slog::Logger;
use tokio::sync::RwLock;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use crate::enforcer::Enforcer;
use crate::error::Error;

use super::PolicyAttachment;

pub struct PolicyAttachmentController {
    logger: Logger,
    manager: ResourceManager<PolicyAttachment>,
    enforcer: Arc<RwLock<Enforcer>>,
}

impl PolicyAttachmentController {
    pub fn new(
        logger: Logger,
        manager: ResourceManager<PolicyAttachment>,
        enforcer: Arc<RwLock<Enforcer>>,
    ) -> Self {
        PolicyAttachmentController {
            logger,
            manager,
            enforcer,
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
