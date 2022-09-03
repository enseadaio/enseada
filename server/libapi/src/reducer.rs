use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use futures::future::BoxFuture;
use futures::{pin_mut, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use cqrs::event::Event;
use cqrs::reducer::Reducer;
use futon::{ChangeEvent, Changes, Database, Envelope, FutonError};

#[derive(Debug, Deserialize, Serialize)]
pub struct Offset {
    pub offset: String,
}

pub struct CouchReducer<A, E, R> {
    id: String,
    db: Database,
    offsets: Database,
    offset: String,
    changes: Changes,
    reducer: R,
    event: PhantomData<E>,
    aggregate: PhantomData<A>,
}

impl<A, E, R> CouchReducer<A, E, R>
where
    A: DeserializeOwned + Serialize + Debug + Default + Send + Sync + 'static,
    E: Event + DeserializeOwned + Send + Sync + 'static,
    E::AggregateId: Display + Debug + Send + Sync,
    R: Reducer<A, E> + Send + Sync + 'static,
{
    pub fn new(
        id: String,
        db: Database,
        offsets: Database,
        offset: String,
        changes: Changes,
        reducer: R,
    ) -> BoxFuture<'static, ()> {
        let actor = Self {
            id,
            db,
            offsets,
            offset,
            changes,
            reducer,
            event: PhantomData::default(),
            aggregate: PhantomData::default(),
        };

        Box::pin(actor.run())
    }

    async fn run(self) {
        let events = self.changes.filter_map(|event| async {
            match event {
                ChangeEvent::Next { doc, seq, .. } => {
                    doc.map(|value| (seq, serde_json::from_value::<E>(value)))
                }
                ChangeEvent::End { .. } => None,
            }
        });

        pin_mut!(events);

        while let Some((offset, event)) = events.next().await {
            if let Err(err) = event {
                tracing::error!(?err, "err");
                continue;
            }
            let event = event.unwrap();
            let res = self.db.get(event.aggregate_id()).await;
            if let Err(err) = res {
                tracing::error!(?err, "failed to fetch aggregate for reduction");
                continue;
            }
            let (id, rev, aggregate) = res
                .unwrap()
                .unwrap_or_else(|| Envelope::new(event.aggregate_id(), A::default()))
                .unwrap_all();
            match self.reducer.apply(aggregate, event) {
                Some(aggregate) => {
                    let envelope = match rev {
                        Some(rev) => {
                            tracing::debug!(%id, %rev, "updating aggregate");
                            Envelope::new_with_rev(id, rev, aggregate)
                        }
                        None => {
                            tracing::debug!(%id, "creating aggregate");
                            Envelope::new(id, aggregate)
                        }
                    };
                    if let Err(err) = self.db.put(envelope).await {
                        match err {
                            FutonError::Conflict(err) => {
                                tracing::warn!(%err, "tried to update stale aggregate");
                            }
                            err => {
                                tracing::error!(%err, "failed to store updated aggregate");
                            }
                        }

                        continue;
                    }
                }
                None => {
                    let rev = rev.unwrap();
                    tracing::debug!(%id, %rev, "deleting aggregate");
                    if let Err(err) = self.db.delete(id, rev).await {
                        tracing::error!(?err, "failed to remove deleted aggregate");
                        continue;
                    }
                }
            }
            let envelope = match self.offsets.get::<&String, Offset>(&self.id).await.unwrap() {
                Some(mut envelope) => {
                    envelope.item_mut().offset = offset;
                    envelope
                }
                None => Envelope::new(self.id.clone(), Offset { offset }),
            };
            self.offsets.put(envelope).await.unwrap();
        }
    }
}
