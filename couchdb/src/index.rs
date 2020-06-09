use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JsonIndex {
    index: serde_json::Value,
    name: String,
    ddoc: Option<String>,
    #[serde(rename = "type")]
    typ: String,
}

impl JsonIndex {
    pub fn new(name: &str, ddoc: Option<String>, index: serde_json::Value) -> Self {
        JsonIndex {
            name: name.to_string(),
            ddoc,
            index,
            typ: "json".to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
