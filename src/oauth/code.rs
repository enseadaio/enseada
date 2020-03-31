pub struct AuthorizationCode {
    code: String
}

impl ToString for AuthorizationCode {
    fn to_string(&self) -> String {
        self.code.clone()
    }
}