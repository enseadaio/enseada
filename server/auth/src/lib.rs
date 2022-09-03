use axum::http::StatusCode;
use axum::routing::get;
use axum::{Extension, Router};
use thiserror::Error;

use futon::{AssertDatabaseParams, Couch, Envelope, FutonError};
use libapi::reducer::{CouchReducer, Offset};
use libapi::ToStatusCode;

use crate::user::{User, UserEvent, UserHandler, UserQuery};

pub mod user;

const OFFSETS_DATABASE: &str = "offsets";
const EVENTS_DATABASE: &str = "users-events";
const AGGREGATE_DATABASE: &str = "users";

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Futon(#[from] FutonError),
}

impl ToStatusCode for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Futon(err) => err.status_code(),
        }
    }
}

pub fn routes(couch: &Couch) -> Router {
    let users = couch.database(AGGREGATE_DATABASE, true);
    let query = UserQuery::new(users);
    let events = couch.database(EVENTS_DATABASE, true);
    let handler = UserHandler::new(events, query.clone());
    Router::new()
        .route("/v1alpha1/users", get(user::list).post(user::create))
        .route("/v1alpha1/users/:id", get(user::get).delete(user::delete))
        .layer(Extension(handler))
        .layer(Extension(query))
}

pub async fn try_init(couch: &Couch) -> Result<(), Error> {
    tracing::info!("Initializing auth module");
    for (name, partitioned) in [
        (OFFSETS_DATABASE, true),
        (EVENTS_DATABASE, true),
        (AGGREGATE_DATABASE, false),
    ] {
        let db = couch.database(name, partitioned);
        db.assert_self(AssertDatabaseParams::default()).await?;
    }

    Ok(())
}

pub async fn start(id: &str, couch: &Couch) -> Result<(), Error> {
    let id = format!("{id}:auth");
    tracing::info!("Starting auth module");
    let events = couch.database(EVENTS_DATABASE, true);
    let changes = events.changes().await?;
    let db = couch.database(AGGREGATE_DATABASE, true);
    let offsets = couch.database(OFFSETS_DATABASE, true);

    let offset = offsets
        .get(&id)
        .await?
        .map(Envelope::unwrap)
        .unwrap_or_else(|| Offset {
            offset: "now".to_string(),
        });

    CouchReducer::new(id, db, offsets, offset.offset, changes, reduce_users).await;
    Ok(())
}

fn reduce_users(mut user: User, event: UserEvent) -> Option<User> {
    match event {
        UserEvent::UserCreated { id, name, ts: _ } => {
            user.id = id;
            user.name = name;
            Some(user)
        }
        UserEvent::UserDeleted { .. } => None,
    }
}
