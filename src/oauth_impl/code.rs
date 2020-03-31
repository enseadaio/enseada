use crate::oauth::code;

pub struct AuthorizationCode;

impl code::AuthorizationCode for AuthorizationCode {

}

impl ToString for AuthorizationCode {
    fn to_string(&self) -> String {
        "code".to_string()
    }
}