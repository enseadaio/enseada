use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct DBInfo {
    pub cluster: DBClusterInfo,
    pub compact_running: bool,
    pub db_name: String,
    pub disk_format_version: usize,
    pub doc_count: usize,
    pub doc_del_count: usize,
    pub instance_start_time: String,
    pub purge_seq: String,
    pub sizes: DBSizes,
    pub update_seq: String,
    pub props: DBProps,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DBSizes {
    active: usize,
    external: usize,
    file: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DBClusterInfo {
    pub n: i16,
    pub q: i16,
    pub r: i16,
    pub w: i16,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DBProps {
    pub partitioned: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Ok {
    pub ok: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PutResponse {
    pub ok: bool,
    pub id: String,
    pub rev: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FindResponse<T> {
    pub docs: Vec<T>,
    pub bookmark: String,
    pub warning: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RowsResponse<T> {
    pub offset: usize,
    pub rows: Vec<RawDocResponse<T>>,
    pub total_rows: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RawDocResponse<T> {
    pub id: String,
    pub key: String,
    pub value: RawDocValue,
    pub doc: Option<T>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum RawDocValue {
    Rev {
        rev: String,
    },
    String(String)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum JsonIndexResultStatus {
    Created,
    Exists,
}

#[derive(Debug, Deserialize)]
pub struct JsonIndexResponse {
    pub result: JsonIndexResultStatus,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Partition {
    pub db_name: String,
    pub partition: String,
    pub doc_count: usize,
    pub doc_del_count: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OkWrapper<T> {
    pub ok: T,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Revs {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev")]
    pub rev: String,
    #[serde(rename = "_deleted", skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
    #[serde(rename = "_revisions")]
    pub revisions: RevisionList,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RevisionList {
    pub start: usize,
    pub ids: Vec<String>,
}
