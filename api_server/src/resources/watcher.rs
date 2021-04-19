use std::convert::TryFrom;
use std::fmt::Debug;

use actix::{Actor, ActorContext, ActorFutureExt, ArbiterHandle, AsyncContext, Context, StreamHandler, Supervised, Supervisor, WrapFuture, Running};
use futures::channel::mpsc;
use futures::SinkExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use slog::Logger;

use api::tonic::{Code, Status};
use api::watch::FromResource;
use api::watch::v1alpha1::EventType;
use couchdb::changes::ChangeEvent;

use crate::resources::id::Id;
use crate::resources::ResourceManager;

pub struct Watcher<T: Debug + Clone + DeserializeOwned + Serialize, E: FromResource<T>> {
    logger: Logger,
    manager: ResourceManager<T>,
    last_seq: String,
    sink: mpsc::Sender<Result<E, Status>>,
}

impl<T: 'static + Debug + Clone + DeserializeOwned + Serialize + Unpin + Send, E: 'static + FromResource<T> + Unpin + Send> Watcher<T, E> {
    pub fn start(logger: Logger, manager: ResourceManager<T>, arbiter: &ArbiterHandle) -> mpsc::Receiver<Result<E, Status>> {
        let (tx, rx) = mpsc::channel(4);
        let w = Watcher {
            logger,
            manager,
            last_seq: "now".to_string(),
            sink: tx,
        };
        Supervisor::start_in_arbiter(arbiter, move |_| w);
        rx
    }

    async fn get_resource(manager: ResourceManager<T>, id: String) -> Result<T, Status> {
        let id = Id::try_from(id).map_err(Status::internal)?;
        manager.get(id.name()).await
    }

    async fn get_deleted_resource(manager: ResourceManager<T>, id: String) -> Result<T, Status> {
        let id = Id::try_from(id).map_err(Status::internal)?;
        manager.get_deleted(id.name()).await
    }
}

impl<T: 'static + Debug + Clone + DeserializeOwned + Serialize + Unpin + Send, E: 'static + FromResource<T> + Unpin + Send> Actor for Watcher<T, E> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let db = self.manager.db();
        let seq = self.last_seq.clone();

        ctx.wait(async move { db.changes_since(seq).await }.into_actor(self)
            .map(|res, this, ctx| {
                match res {
                    Ok(s) => {
                        Self::add_stream(s, ctx);
                    }
                    Err(err) => {
                        slog::error!(this.logger, "{}", err);
                        ctx.stop();
                    }
                };
            }));
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        // If we stop we drop subscribers. We don't want that.
        Running::Continue
    }
}

impl<T: 'static + Debug + Clone + DeserializeOwned + Serialize + Unpin + Send, E: 'static + FromResource<T> + Unpin + Send> Supervised for Watcher<T, E> {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        slog::info!(self.logger, "Restarting watcher")
    }
}

impl<T: 'static + Debug + Clone + DeserializeOwned + Serialize + Unpin + Send, E: 'static + FromResource<T> + Unpin + Send> StreamHandler<ChangeEvent> for Watcher<T, E> {
    fn handle(&mut self, item: ChangeEvent, ctx: &mut Self::Context) {
        slog::debug!(self.logger, "Handling event {:?}", item);
        match item {
            ChangeEvent::Next { id, deleted, .. } => {
                let event_type = if deleted.unwrap_or(false) { EventType::Deleted } else { EventType::Changed };
                let logger = self.logger.clone();
                let mgr = self.manager.clone();
                let mut sink = self.sink.clone();
                ctx.wait(async move {
                    let res = match event_type {
                        EventType::Changed => Self::get_resource(mgr, id).await,
                        EventType::Deleted => Self::get_deleted_resource(mgr, id).await,
                    };
                    if let Err(err) = &res {
                        if err.code() == Code::Internal {
                            slog::error!(logger, "{}", err);
                        }
                    }
                    slog::debug!(logger, "Sending event {:?} for resource {:?}", event_type, res);
                    sink.send(res.map(|res| E::from_res(event_type, res))).await.expect("failed to send resource event");
                }.into_actor(self));
            }
            ChangeEvent::End { last_seq, .. } => {
                self.last_seq = last_seq;
                Actor::started(self, ctx);
            }
        }
    }
}
