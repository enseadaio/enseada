pub mod v1beta1 {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)]
    pub struct UserModel {
        pub username: String,
        pub enabled: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct UserPost {
        pub username: String,
        pub password: String,
        #[serde(default)]
        pub roles: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct UserPut {
        pub enabled: Option<bool>,
    }
}
