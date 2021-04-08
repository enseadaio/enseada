pub mod v1beta1 {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use enseada::expiration::Expiration;
    use oauth::scope::Scope;

    #[derive(Debug, Serialize, PartialEq)]
    pub struct PersonalAccessTokenModel {
        pub id: String,
        pub label: String,
        pub client_id: String,
        pub scope: Scope,
        pub user_id: Option<String>,
        pub expiration: Expiration,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub revoked_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct CreatedPersonalAccessToken {
        pub access_token: String,
        #[serde(flatten)]
        pub pat: PersonalAccessTokenModel,
    }

    #[derive(Debug, Deserialize)]
    pub struct PersonalAccessTokenPost {
        pub label: String,
        pub scope: Scope,
        pub expiration: Option<DateTime<Utc>>,
    }
}
