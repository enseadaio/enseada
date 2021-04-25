use std::convert::TryFrom;
use std::time::Duration;

use actix::{Actor, ActorContext, ActorFutureExt, ArbiterHandle, AsyncContext, Context, StreamHandler, Supervised, Supervisor, WrapFuture};
use futures::channel::mpsc;
use futures::SinkExt;
use slog::Logger;

use api::core::v1alpha1::Event;
use api::Resource;
use couchdb::changes::ChangeEvent;

use crate::ControllerError;
use crate::id::Id;
use crate::manager::ResourceManager;

pub struct Watcher<T: Resource> {
    logger: Logger,
    manager: ResourceManager<T>,
    last_seq: String,
    sink: mpsc::Sender<Result<Event<T>, ControllerError>>,
    tick: Duration,
}

impl<T: 'static + Resource + Unpin> Watcher<T> {
    pub fn start(logger: Logger, manager: ResourceManager<T>, arbiter: &ArbiterHandle) -> mpsc::Receiver<Result<Event<T>, ControllerError>> {
        let (tx, rx) = mpsc::channel(4);
        let w = Watcher {
            logger,
            manager,
            last_seq: "0".to_string(),
            sink: tx,
            tick: Duration::from_secs(30),
        };
        Supervisor::start_in_arbiter(arbiter, move |_| w);
        rx
    }

    async fn get_resource(manager: ResourceManager<T>, id: String) -> Result<T, ControllerError> {
        let id = Id::try_from(id).map_err(ControllerError::from)?;
        manager.get(id.name()).await
    }

    fn start_db_stream(&mut self, ctx: &mut Context<Self>) {
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

    fn start_tick_stream(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(self.tick, |this, ctx| {
            let manager = this.manager.clone();
            ctx.wait(async move { manager.list().await}.into_actor(this)
                .map(|res, this, ctx| {
                    match res {
                        Ok(list) => {
                            for resource in list {
                                let mut sink = this.sink.clone();
                                ctx.wait(async move {
                                    sink.send(Ok(Event::from(resource))).await.expect("failed to send resource event");
                                }.into_actor(this));
                            }
                        }
                        Err(err) => {
                            slog::error!(this.logger, "{}. Retrying in {} seconds", err, this.tick.as_secs());
                        }
                    };
                }));
        });
    }
}

impl<T: 'static + Resource + Unpin + Send> Actor for Watcher<T> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_db_stream(ctx);
        self.start_tick_stream(ctx);
    }
}

impl<T: 'static + Resource + Unpin + Send> Supervised for Watcher<T> {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        slog::warn!(self.logger, "Restarting watcher")
    }
}

impl<T: 'static + Resource + Unpin + Send> StreamHandler<ChangeEvent> for Watcher<T> {
    fn handle(&mut self, item: ChangeEvent, ctx: &mut Self::Context) {
        slog::trace!(self.logger, "Handling event {:?}", item);
        match item {
            ChangeEvent::Next { id, .. } => {
                let logger = self.logger.clone();
                let mgr = self.manager.clone();
                let mut sink = self.sink.clone();
                ctx.wait(async move {
                    let res = Self::get_resource(mgr, id).await;
                    if let Err(err) = &res {
                        slog::error!(logger, "{}", err);
                        return;
                    }
                    slog::trace!(logger, "Sending event for resource {:?}", res);
                    sink.send(res.map(Event::from)).await.expect("failed to send resource event");
                }.into_actor(self));
            }
            ChangeEvent::End { last_seq, .. } => {
                self.last_seq = last_seq;
                Actor::started(self, ctx);
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {}
}

