use slog::Logger;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use super::Policy;

pub struct PolicyController {
    logger: Logger,
    manager: ResourceManager<Policy>,
}

impl PolicyController {
    pub fn new(
        logger: Logger,
        manager: ResourceManager<Policy>,
    ) -> Self {
        PolicyController {
            logger,
            manager,
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
        }
        Ok(())
    }
}
