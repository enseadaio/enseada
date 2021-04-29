use std::sync::Arc;

use slog::Logger;

use controller_runtime::{
    async_trait, ControllerError, Reconciler, ReconciliationError, ResourceManager,
};

use crate::enforcer::Enforcer;
use crate::policy::v1alpha1::Policy;
use tokio::sync::RwLock;

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

/*
apiVersion: rbac/v1alpha1
kind: Policy
metadata:
    name: test
rules:
- resources: ['* / * / *']
  actions: ['*']
---
apiVersion: rbac/v1alpha1
kind: PolicyAssignment
metadata:
    name: test-role
spec:
  policyRef:
    name: test
  subjects:
    - name: test-role
      kind: Role
    - name: test-user
      kind: User
*/
#[async_trait]
impl Reconciler<Policy> for PolicyController {
    async fn reconcile(
        &mut self,
        _resource: Policy,
    ) -> Result<(), ReconciliationError<ControllerError>> {
        let mut enforcer = self.enforcer.write().await;
        enforcer.load_rules();

        Ok(())
    }
}
