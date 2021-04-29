use std::sync::Arc;

use slog::Logger;
use tokio::sync::RwLock;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use crate::enforcer::Enforcer;
use crate::error::Error;

use super::Policy;

pub struct PolicyController {
    logger: Logger,
    manager: ResourceManager<Policy>,
    enforcer: Arc<RwLock<Enforcer>>,
}

impl PolicyController {
    pub fn new(
        logger: Logger,
        manager: ResourceManager<Policy>,
        enforcer: Arc<RwLock<Enforcer>>,
    ) -> Self {
        PolicyController {
            logger,
            manager,
            enforcer,
        }
    }
}

#[async_trait]
impl Reconciler<Policy> for PolicyController {
    async fn reconcile(
        &mut self,
        mut policy: Policy,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        if policy.metadata.is_just_created() {
            policy.metadata.created_at = Some(Utc::now());
            let name = policy.metadata.name.clone();
            self.manager.put(&name, policy).await?;
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
