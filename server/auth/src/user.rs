use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Error;
use futon::{Database, Envelope};
use libapi::cqrs::command::{CommandHandler, ValidationErrors};
use libapi::cqrs::event::Event;
use libapi::id::Id;
use libapi::pagination::PaginationQuery;
use libapi::{ApiError, ApiResult, OffsetDateTime};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct User {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UserCreateCommand {
    name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UserDeleteCommand {
    id: Id,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum UserEvent {
    UserCreated {
        id: Id,
        name: String,
        ts: OffsetDateTime,
    },
    UserDeleted {
        id: Id,
        ts: OffsetDateTime,
    },
}

impl Event for UserEvent {
    type AggregateId = Id;

    fn aggregate_id(&self) -> Self::AggregateId {
        match self {
            Self::UserCreated { id, .. } => *id,
            Self::UserDeleted { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> OffsetDateTime {
        match self {
            Self::UserCreated { ts, .. } => *ts,
            Self::UserDeleted { ts, .. } => *ts,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserHandler {
    db: Database,
    query: UserQuery,
}

impl UserHandler {
    pub fn new(db: Database, query: UserQuery) -> Self {
        Self { db, query }
    }
}

impl UserHandler {
    async fn store(&self, id: Id, events: Vec<UserEvent>) -> Result<(), Error> {
        let events = events
            .into_iter()
            .map(|event| Envelope::new(format!("{}:{}", id, libapi::now()), event))
            .collect();
        self.db.bulk_insert(events).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UserQuery {
    db: Database,
}

#[derive(Debug, Serialize)]
pub struct NameSelector {
    name: String,
}

impl UserQuery {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self, limit: usize, offset: usize) -> Result<Vec<User>, Error> {
        let res = self.db.list(limit, offset).await?;
        tracing::debug!(?res, "res");
        let users = res
            .rows
            .into_iter()
            .map(|row| row.doc.unwrap())
            .map(Envelope::unwrap)
            .collect();
        tracing::debug!("listing users");
        Ok(users)
    }

    pub async fn get(&self, id: Id) -> Result<Option<User>, Error> {
        let user = self.db.get(id).await?.map(Envelope::unwrap);
        Ok(user)
    }

    pub async fn find_by_name(&self, name: impl ToString) -> Result<Option<User>, Error> {
        let user = self.db.find_by(NameSelector { name: name.to_string() }).await?.map(Envelope::unwrap);
        Ok(user)
    }
}

#[libapi::cqrs::async_trait]
impl CommandHandler<UserCreateCommand, UserEvent> for UserHandler {
    async fn validate(
        &self,
        command: UserCreateCommand,
    ) -> Result<(Id, UserCreateCommand), ValidationErrors> {
        let user = self.query.find_by_name(&command.name).await.map_err(|err| vec![err.to_string().into()].into())?;
        match user {
            Some(_) => Err(vec!["user already exists".into()].into()),
            None => Ok((Id::new(), command)),
        }
    }

    fn apply(&self, id: Id, command: UserCreateCommand) -> Vec<UserEvent> {
        vec![UserEvent::UserCreated {
            id,
            name: command.name,
            ts: OffsetDateTime::now_utc(),
        }]
    }
}

#[libapi::cqrs::async_trait]
impl CommandHandler<UserDeleteCommand, UserEvent> for UserHandler {
    async fn validate(
        &self,
        command: UserDeleteCommand,
    ) -> Result<(Id, UserDeleteCommand), ValidationErrors> {
        // todo validate existence
        Ok((command.id, command))
    }

    fn apply(&self, id: Id, _command: UserDeleteCommand) -> Vec<UserEvent> {
        vec![UserEvent::UserDeleted {
            id,
            ts: OffsetDateTime::now_utc(),
        }]
    }
}

pub(crate) async fn list(
    Extension(query): Extension<UserQuery>,
    Query(page): Query<PaginationQuery>,
) -> ApiResult<Json<Vec<User>>> {
    let users = query.list(page.limit, page.offset).await?;
    Ok(Json(users))
}

pub(crate) async fn get(
    Extension(query): Extension<UserQuery>,
    Path(id): Path<Id>,
) -> ApiResult<Json<User>> {
    let user = query.get(id).await?;
    match user {
        Some(user) => Ok(Json(user)),
        None => Err(ApiError::not_found(format!("user '{}' not found", id))),
    }
}

pub(crate) async fn create(
    Extension(handler): Extension<UserHandler>,
    Json(cmd): Json<UserCreateCommand>,
) -> ApiResult<StatusCode> {
    tracing::info!("creating {:?}", cmd);
    let (id, cmd) = handler.validate(cmd).await?;
    let events = handler.apply(id, cmd);
    handler.store(id, events).await?;
    Ok(StatusCode::ACCEPTED)
}

pub(crate) async fn delete(
    Extension(handler): Extension<UserHandler>,
    Path(id): Path<Id>,
) -> ApiResult<StatusCode> {
    let cmd = UserDeleteCommand { id };
    tracing::info!("deleting {:?}", cmd);
    let (id, cmd) = handler.validate(cmd).await?;
    let events = handler.apply(id, cmd);
    handler.store(id, events).await?;
    Ok(StatusCode::ACCEPTED)
}
