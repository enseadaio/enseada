use slog::Logger;

use controller_runtime::{async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager, Utc};

use super::{User, UserStatus};

pub struct UserController {
    logger: Logger,
    manager: ResourceManager<User>,
}

impl UserController {
    pub fn new(logger: Logger, manager: ResourceManager<User>) -> Self {
        UserController { logger, manager }
    }
}

#[async_trait]
impl Reconciler<User> for UserController {
    async fn reconcile(
        &mut self,
        mut user: User,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        let mut dirty = false;

        if user.metadata.is_just_created() {
            user.metadata.created_at = Some(Utc::now());
            dirty = true;
        }

        if user.metadata.is_deleted() {
            return Ok(());
        }

        let enabled = user.spec.enabled;
        dirty = user
            .status
            .as_ref()
            .map_or(dirty, |status| status.enabled != enabled);
        if dirty {
            user.status = Some(UserStatus { enabled });
            let name = user.metadata.name.clone();
            self.manager.put(&name, user).await?;
        }
        Ok(())
    }
}
