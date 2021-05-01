use serde::{Deserialize, Serialize};

use api::Resource;
use api::core::v1alpha1::{Metadata, TypeMeta};
pub use controller::UserController;

mod controller;

#[derive(Clone, Debug, Deserialize, Serialize, Resource)]
#[serde(rename_all = "camelCase")]
#[resource(api_version = "auth/v1alpha1", kind = "User", kind_plural = "users", status = "UserStatus")]
pub struct User {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: Metadata,
    pub spec: UserSpec,
    pub status: Option<UserStatus>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSpec {
    pub enabled: bool,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatus {
    pub enabled: bool,
}
