

use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use schemars::JsonSchema;

use serde::{Deserialize, Serialize};
use std::fmt::{Debug};

use time::format_description::well_known::Rfc3339;

pub use time::OffsetDateTime;

pub use crate::error::{ApiError, ToStatusCode};

mod error;
pub mod id;
pub mod pagination;
pub mod reducer;

pub use cqrs;




pub trait Resource<Spec> {
    fn meta(&self) -> &Metadata;
    fn meta_mut(&mut self) -> &mut Metadata;
    fn spec(&self) -> &Spec;
    fn spec_mut(&mut self) -> &mut Spec;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    name: String,
    revision: Option<String>,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
}

impl JsonSchema for Metadata {
    fn schema_name() -> String {
        "Metadata".into()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        Schema::Bool(true) // TODO
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

pub fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("failed to format UTC Now in RFC3339")
}
