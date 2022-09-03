use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Stream, StreamExt};
use hyper::{Body, Request};
use serde::Deserialize;
use serde_json::Value;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;

use crate::{Client, FutonResult};

#[derive(Debug)]
pub struct Changes {
    receiver: ReceiverStream<ChangeEvent>,
    handle: JoinHandle<()>,
}

impl Changes {
    pub fn new<F>(client: Client, request: F) -> Self
    where
        F: Fn(String) -> Request<Body> + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel(100);

        let handle = tokio::task::spawn(async move {
            let res: FutonResult<()> = async move {
                let mut since = "now".to_string();
                loop {
                    let req = request(since.clone());
                    let mut body = client.raw_execute(req).await?.into_body();
                    while let Some(chunk) = body.next().await {
                        let mut lines = Cursor::new(chunk?).lines();
                        while let Some(line) = lines.next_line().await? {
                            if line.is_empty() {
                                continue;
                            }

                            let event = serde_json::from_str(&line)?;
                            match &event {
                                ChangeEvent::End { .. } => continue,
                                ChangeEvent::Next { seq, .. } => {
                                    since = seq.clone();
                                    let _ = sender.send(event).await;
                                }
                            }
                        }
                    }
                }
            }
            .await;

            if let Err(err) = res {
                tracing::error!(
                    ?err,
                    "failure while processing changes feed in background process"
                );
            }
        });

        Self {
            receiver: ReceiverStream::new(receiver),
            handle,
        }
    }
}

impl Stream for Changes {
    type Item = ChangeEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}

impl Drop for Changes {
    fn drop(&mut self) {
        self.handle.abort()
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ChangeEvent {
    Next {
        id: String,
        seq: String,
        changes: Vec<Change>,
        deleted: Option<bool>,
        doc: Option<Value>,
    },
    End {
        last_seq: String,
        pending: u64,
    },
}

impl ChangeEvent {
    fn is_end(&self) -> bool {
        matches!(self, Self::End { .. })
    }
}

#[derive(Debug, Deserialize)]
pub struct Change {
    rev: String,
}
