use slog::Logger;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use super::{OAuthClient, OAuthClientStatus};
use crate::api::v1alpha1::OAuthClientCondition;

pub struct OAuthClientController {
    logger: Logger,
    manager: ResourceManager<OAuthClient>,
}

impl OAuthClientController {
    pub fn new(logger: Logger, manager: ResourceManager<OAuthClient>) -> Self {
        OAuthClientController { logger, manager }
    }

    pub fn reconcile_public_client(&mut self, dirty: bool, mut status: OAuthClientStatus) -> Result<(bool, OAuthClientStatus), ControllerError> {
        let res = match status.condition {
            OAuthClientCondition::Pending => {
                status.condition = OAuthClientCondition::Active;
                (true, status)
            },
            _ => (dirty, status),
        };

        Ok(res)
    }
}

#[async_trait]
impl Reconciler<OAuthClient> for OAuthClientController {
    async fn reconcile(
        &mut self,
        mut client: OAuthClient,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        let mut dirty = false;

        if client.metadata.is_just_created() {
            client.metadata.created_at = Some(Utc::now());
            dirty = true;
        }

        if client.metadata.is_deleted() {
            return Ok(());
        }

        let status = client.status.unwrap_or_else(OAuthClientStatus::default);

        let (dirty, status) = self.reconcile_public_client(dirty, status)?;

        if dirty {
            client.status = Some(status);
            let name = client.metadata.name.clone();
            self.manager.put(&name, client).await?;
        }
        Ok(())
    }
}
