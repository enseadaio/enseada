pub mod v1beta1 {
    use serde::{Deserialize, Serialize};

    use enseada::guid::Guid;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PermissionModel {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub subject: Option<Guid>,
        pub object: Guid,
        pub action: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct RoleModel {
        pub role: String,
    }

    #[derive(Debug, Serialize)]
    pub struct UserCapabilities {
        pub permissions: Vec<PermissionModel>,
        pub roles: Vec<String>,
    }
}
