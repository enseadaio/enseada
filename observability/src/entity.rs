use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Status {
    Healty,
    Unhealthy(String),
}
