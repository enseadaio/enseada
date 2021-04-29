use std::sync::Arc;

use slog::Logger;
use tokio::sync::RwLock;

use controller_runtime::{
    async_trait, ControllerError, Reconciler, ReconciliationError,
};

use crate::enforcer::Enforcer;

use super::Policy;
use crate::error::Error;

pub struct PolicyController {
    logger: Logger,
    enforcer: Arc<RwLock<Enforcer>>,
}

impl PolicyController {
    pub fn new(
        logger: Logger,
        enforcer: Arc<RwLock<Enforcer>>,
    ) -> Self {
        PolicyController {
            logger,
            enforcer,
        }
    }
}

#[async_trait]
impl Reconciler<Policy> for PolicyController {
    async fn reconcile(
        &mut self,
        _resource: Policy,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        let mut enforcer = self.enforcer.write().await;
        if let Err(Error::Controller(err)) = enforcer.load_model_from_resources().await {
            slog::error!(self.logger, "Failed to reload ACL engine: {}", err);
            Err(ReconciliationError::wrap(err))
        } else {
            Ok(())
        }
    }
}
