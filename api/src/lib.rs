pub mod pat;
pub mod rbac;
pub mod users;

pub static SPEC_YML: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi.yml"));
