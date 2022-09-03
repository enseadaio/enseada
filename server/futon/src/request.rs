use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FindRequest<S> {
    pub selector: S,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub skip: usize,
}


