use std::time::Duration;

use actix::{Actor, ActorFutureExt, ArbiterHandle, AsyncContext, Context, StreamHandler, Supervised, Supervisor, WrapFuture};
use futures::StreamExt;
use slog::Logger;

use couchdb::db::Database;

use crate::ResourceWrapper;

pub struct GarbageCollector {
    logger: Logger,
    db: Database,
    tick: Duration,
}

impl GarbageCollector {
    pub fn start(logger: Logger, db: Database, arbiter: &ArbiterHandle, polling_interval: Duration) {
        let gc = Self {
            logger,
            db,
            tick: polling_interval,
        };
        Supervisor::start_in_arbiter(arbiter, move |_| gc);
    }

    fn start_tick_stream(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(self.tick, |this, ctx| {
            slog::trace!(this.logger, "Running GC");
            let db = this.db.clone();
            ctx.wait(async move { db.stream::<ResourceWrapper<serde_json::Value>>() }.into_actor(this)
                .map(|s, _, ctx| {
                    Self::add_stream(s.filter_map(|res| async move { res.ok() }), ctx);
                }));
        });
    }
}

impl Actor for GarbageCollector {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_tick_stream(ctx);
    }
}

impl Supervised for GarbageCollector {}

impl StreamHandler<ResourceWrapper<serde_json::Value>> for GarbageCollector {
    fn handle(&mut self, wrapper: ResourceWrapper<serde_json::Value>, ctx: &mut Self::Context) {
        ctx.wait(async move { wrapper }.into_actor(self)
            .map(|wrapper, this, ctx| {
                let id = wrapper.id().to_string();
                let rev = wrapper.rev().unwrap().to_string();
                let res = wrapper.into_inner();
                let has_finalizers = res["metadata"]["finalizers"].as_array().map(Vec::is_empty).unwrap_or(false);
                if !has_finalizers {
                    let db = this.db.clone();
                    ctx.wait(async move { db.delete(&id, &rev).await }.into_actor(this).map(|res, this, _| {
                        if let Err(err) = res {
                            slog::error!(this.logger, "{}", err);
                        }
                    }));
                }
            }))
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {}
}
