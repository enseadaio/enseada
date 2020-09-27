/**
    Huge shout-out to Luca Palmieri (https://github.com/LukeMathWalker)

    He made this code work by bending the type system to his will
*/
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use actix::{Actor, Arbiter, Context, Handler, Message, Recipient, Supervised, Supervisor};
use async_trait::async_trait;

pub use events_derive::Event;

pub trait Event: 'static + Send + Sync + Debug {}

#[derive(Message)]
#[rtype(result = "()")]
struct EventMessage<E: Event>(Arc<E>);

#[derive(Default)]
pub struct EventBus {
    subscribers: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
    arbiter: Arbiter,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn broadcast<E: Event>(&self, e: E) {
        let event_id = TypeId::of::<E>();
        if let Some(subs) = self.subscribers.get(&event_id) {
            let event = Arc::new(e);
            for sub in subs {
                let sub: &Recipient<EventMessage<E>> =
                    (&**sub as &(dyn Any)).downcast_ref().unwrap();
                let msg = EventMessage(event.clone());
                sub.do_send(msg).unwrap();
            }
        }
    }

    pub fn subscribe<E: Event, H: 'static + EventHandler<E>>(&mut self, h: H) {
        let a = Arc::new(h);
        let sub = Subscriber(a);
        let sub = Supervisor::start_in_arbiter(&self.arbiter, |_ctx| sub).recipient();
        let event_id = TypeId::of::<E>();
        let subs = self.subscribers.entry(event_id).or_insert_with(Vec::new);
        let sub = Box::new(sub);
        let sub = sub as Box<dyn Any + Send + Sync>;
        subs.push(sub);
    }
}

impl Debug for EventBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventBus").finish()
    }
}

#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    async fn handle(&self, event: &E);
}

#[async_trait]
impl<E: Event, H: EventHandler<E>> EventHandler<E> for Arc<H> {
    async fn handle(&self, e: &E) {
        self.as_ref().handle(e).await
    }
}

#[async_trait]
impl<E: Event, H: EventHandler<E>> EventHandler<E> for Box<H> {
    async fn handle(&self, e: &E) {
        self.as_ref().handle(e).await
    }
}

struct Subscriber<E: Event>(Arc<dyn EventHandler<E>>);

impl<E: Event> Actor for Subscriber<E> {
    type Context = Context<Self>;
}

impl<E: Event> Supervised for Subscriber<E> {}

impl<E: Event> Handler<EventMessage<E>> for Subscriber<E> {
    type Result = Pin<Box<dyn Future<Output = ()>>>;

    fn handle(
        &mut self,
        EventMessage(e): EventMessage<E>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let h = self.0.clone();
        Box::pin(async move { h.handle(e.as_ref()).await })
    }
}
