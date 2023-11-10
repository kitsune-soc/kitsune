//!
//! Redis implementation
//!

use crate::{util::TransparentDebug, MessagingBackend, Result};
use ahash::AHashMap;
use futures_util::{future, Stream, StreamExt, TryStreamExt};
use kitsune_retry_policies::{futures_backoff_policy, RetryFutureExt};
use redis::{
    aio::{ConnectionManager, PubSub},
    AsyncCommands, RedisError,
};
use std::fmt::Debug;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;

const BROADCAST_CAPACITY: usize = 10;
const REGISTRATION_QUEUE_SIZE: usize = 50;

macro_rules! handle_err {
    ($result:expr, $msg:literal $(,)?) => {{
        if let Err(error) = { $result } {
            error!(?error, $msg);
        }
    }};
    ($result:expr $(,)?) => {
        handle_err!($result, "");
    };
}

#[derive(Debug)]
struct RegistrationMessage {
    channel_pattern: String,
    responder: oneshot::Sender<broadcast::Receiver<Vec<u8>>>,
}

struct MultiplexActor {
    client: redis::Client,
    conn: PubSub,
    mapping: AHashMap<String, broadcast::Sender<Vec<u8>>>,
    registration_queue: mpsc::Receiver<RegistrationMessage>,
}

impl MultiplexActor {
    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.registration_queue.recv() => {
                    let receiver = if let Some(sender) = self.mapping.get(&msg.channel_pattern) {
                        sender.subscribe()
                    } else {
                        let (sender, receiver) = broadcast::channel(BROADCAST_CAPACITY);

                        handle_err!(self.conn.psubscribe(
                            msg.channel_pattern.as_str()).await,
                            "Failed to subscribe to pattern",
                        );

                        self.mapping.insert(msg.channel_pattern, sender);
                        receiver
                    };
                    drop(msg.responder.send(receiver));
                }
                msg = future::poll_fn(|ctx| self.conn.on_message().poll_next_unpin(ctx)) => {
                    if let Some(msg) = msg {
                        let pattern: String = msg.get_pattern().unwrap();

                        if let Some(sender) = self.mapping.get(&pattern) {
                            if sender.send(msg.get_payload_bytes().to_vec()).is_err() {
                                // According to the tokio docs, this case only occurs when all receivers have been dropped
                                handle_err!(
                                    self.conn.punsubscribe(pattern.as_str()).await,
                                    "Failed to unsubscribe from pattern",
                                );
                                self.mapping.remove(&pattern);
                            }
                        } else {
                            debug!(%pattern, "Failed to find correct receiver");
                        }
                    } else {
                        self.conn = (|| {
                            let client = self.client.clone();
                            async move {
                                client
                                    .get_async_connection()
                                    .await
                                    .map(|conn| TransparentDebug(conn.into_pubsub()))
                            }
                        })
                        .retry(futures_backoff_policy())
                        .await
                        .map(|conn| conn.0)
                        .unwrap();

                        for key in self.mapping.keys() {
                            handle_err!(
                                self.conn.psubscribe(key).await,
                                "Failed to subscribe to pattern",
                            );
                        }
                    }
                }
            }
        }
    }

    pub async fn spawn(
        client: redis::Client,
    ) -> Result<mpsc::Sender<RegistrationMessage>, RedisError> {
        let (sender, receiver) = mpsc::channel(REGISTRATION_QUEUE_SIZE);

        let actor = Self {
            mapping: AHashMap::new(),
            conn: client.get_async_connection().await?.into_pubsub(),
            client,
            registration_queue: receiver,
        };
        tokio::spawn(actor.run());

        Ok(sender)
    }
}

/// Implementation of the [`MessagingBackend`] trait for Redis PubSub
///
/// Note: Channel names, when passed to the `message_stream` function, are interpreted as channel patterns.
pub struct RedisMessagingBackend {
    pub_connection: ConnectionManager,
    sub_actor: mpsc::Sender<RegistrationMessage>,
}

impl RedisMessagingBackend {
    /// Create a new Redis PubSub backend
    ///
    /// # Errors
    ///
    /// - Failed to connect to the Redis instance
    pub async fn new(conn_string: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(conn_string)?;
        let sub_actor = MultiplexActor::spawn(client.clone()).await?;
        let pub_connection = ConnectionManager::new(client).await?;

        Ok(Self {
            pub_connection,
            sub_actor,
        })
    }
}

impl MessagingBackend for RedisMessagingBackend {
    async fn enqueue(&self, channel_name: &str, message: Vec<u8>) -> Result<()> {
        self.pub_connection
            .clone()
            .publish(channel_name, message)
            .await
            .map_err(Into::into)
    }

    async fn message_stream(
        &self,
        channel_name: String,
    ) -> Result<impl Stream<Item = Result<Vec<u8>>> + 'static> {
        let (sender, receiver) = oneshot::channel();
        self.sub_actor
            .send(RegistrationMessage {
                channel_pattern: channel_name,
                responder: sender,
            })
            .await?;
        let broadcast_receiver = receiver.await?;

        Ok(BroadcastStream::new(broadcast_receiver).map_err(Into::into))
    }
}
