use slog::Logger;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use super::{OAuthAuthorizationCode};

pub struct OAuthAuthorizationCodeController {
    logger: Logger,
    manager: ResourceManager<OAuthAuthorizationCode>,
}

impl OAuthAuthorizationCodeController {
    pub fn new(logger: Logger, manager: ResourceManager<OAuthAuthorizationCode>) -> Self {
        OAuthAuthorizationCodeController { logger, manager }
    }
}

#[async_trait]
impl Reconciler<OAuthAuthorizationCode> for OAuthAuthorizationCodeController {
    async fn reconcile(
        &mut self,
        _code: OAuthAuthorizationCode,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        Ok(())
    }
}
