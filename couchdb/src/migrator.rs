use std::collections::HashMap;

use serde::Deserialize;
use snafu::{ResultExt, Snafu};

use crate::design_document::ViewDoc;
use crate::error::Error;
use crate::index::JsonIndex;
use crate::Couch;

#[derive(Debug, Snafu)]
pub enum MigrationError {
    #[snafu(display("Failed to read migration: {}", source))]
    DeserializationError { source: serde_json::Error },
    #[snafu(display("Failed to run migration {}: view file {} not found", op, file))]
    MissingViewFileError { file: String, op: String },
    #[snafu(display("Failed to run migration {}: {}", op, source))]
    RunError { op: String, source: Error },
}

impl From<serde_json::Error> for MigrationError {
    fn from(err: serde_json::Error) -> Self {
        MigrationError::DeserializationError { source: err }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MigrationOperation {
    CreateDatabase {
        name: String,
        partitioned: bool,
    },
    CreateIndex {
        name: String,
        database: String,
        index: serde_json::Value,
        design_doc: Option<String>,
    },
    CreateView {
        name: String,
        database: String,
        design_doc: String,
        map_file: String,
        reduce_file: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct Migration {
    name: String,
    operations: Vec<MigrationOperation>,
}

pub struct Migrator<'c> {
    client: &'c Couch,
    migrations: Vec<Migration>,
    scripts: HashMap<String, String>,
}

impl<'c> Migrator<'c> {
    pub fn new(
        client: &'c Couch,
        migrations: Vec<String>,
        scripts: HashMap<String, String>,
    ) -> Result<Self, MigrationError> {
        let mut migs = Vec::new();
        for mig in &migrations {
            let mig = serde_json::from_str(mig)?;
            migs.push(mig);
        }
        Ok(Migrator {
            client,
            migrations: migs,
            scripts,
        })
    }

    pub async fn run(&self) -> Result<(), MigrationError> {
        log::debug!("Running CouchDB migrations");
        if self.migrations.is_empty() {
            log::debug!("No migrations found. Nothing to do");
            return Ok(());
        }

        for mig in &self.migrations {
            log::debug!("Running '{}' migration", mig.name);

            for op in &mig.operations {
                match op {
                    MigrationOperation::CreateDatabase { name, partitioned } => {
                        self.create_db(name, *partitioned, mig.name.clone()).await?
                    }
                    MigrationOperation::CreateIndex {
                        name,
                        database,
                        index,
                        design_doc,
                    } => {
                        self.create_index(
                            name,
                            database,
                            design_doc.clone(),
                            index.clone(),
                            mig.name.clone(),
                        )
                        .await?
                    }
                    MigrationOperation::CreateView {
                        name,
                        database,
                        design_doc,
                        map_file,
                        reduce_file,
                    } => {
                        self.create_view(
                            name,
                            database,
                            design_doc,
                            map_file,
                            reduce_file.as_deref(),
                            mig.name.clone(),
                        )
                        .await?
                    }
                }
            }
        }
        Ok(())
    }

    async fn create_db(
        &self,
        name: &str,
        partitioned: bool,
        op: String,
    ) -> Result<(), MigrationError> {
        log::debug!("Creating database {}", name);
        let db = self.client.database(name, partitioned);
        if db.get_self().await.is_ok() {
            log::debug!("Database {} already exists. Skipping", db.name());
            return Ok(());
        }

        if !db.create_self().await.context(RunError { op })? {
            log::warn!("Database creation returned ok: false");
        }

        Ok(())
    }

    async fn create_index(
        &self,
        name: &str,
        database: &str,
        ddoc: Option<String>,
        index: serde_json::Value,
        op: String,
    ) -> Result<(), MigrationError> {
        let db = self.client.database(database, true);

        let index = JsonIndex::new(name, ddoc, index);
        let name = index.name().to_string();
        if !db.create_index(index).await.context(RunError { op })? {
            log::debug!("Index {} already exists. Skipping", name);
        }

        Ok(())
    }

    async fn create_view(
        &self,
        name: &str,
        database: &str,
        ddoc: &str,
        map_file: &str,
        reduce_file: Option<&str>,
        op: String,
    ) -> Result<(), MigrationError> {
        let db = self.client.database(database, true);

        let map_fun =
            self.scripts
                .get(map_file)
                .ok_or_else(|| MigrationError::MissingViewFileError {
                    file: map_file.to_string(),
                    op: op.clone(),
                })?;
        let view = if let Some(reduce_file) = reduce_file {
            let reduce_fun = self.scripts.get(reduce_file).ok_or_else(|| {
                MigrationError::MissingViewFileError {
                    file: reduce_file.to_string(),
                    op: op.clone(),
                }
            })?;
            ViewDoc::from_map_reduce(name, map_fun, reduce_fun)
        } else {
            ViewDoc::from_map(name, map_fun)
        };

        db.create_view(ddoc, view).await.context(RunError { op })?;
        Ok(())
    }
}
