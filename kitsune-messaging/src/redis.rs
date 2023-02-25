//!
//! Redis implementation
//!

use crate::{MessagingBackend, Result};
use async_trait::async_trait;
use futures_util::{future, stream::BoxStream, StreamExt, TryStreamExt};
use redis::{
    aio::{ConnectionManager, PubSub},
    AsyncCommands, RedisError,
};
use std::{collections::HashMap, time::Duration};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;

const BROADCAST_CAPACITY: usize = 10;
const CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(5);
const REGISTRATION_QUEUE_SIZE: usize = 50;

#[derive(Debug)]
struct RegistrationMessage {
    channel_pattern: String,
    responder: oneshot::Sender<broadcast::Receiver<Vec<u8>>>,
}

struct MultiplexActor {
    mapping: HashMap<String, broadcast::Sender<Vec<u8>>>,
    pubsub_conn: PubSub,
    redis_client: redis::Client,
    registration_queue: mpsc::Receiver<RegistrationMessage>,
}

impl MultiplexActor {
    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.registration_queue.recv() => {
                    if let Some(sender) = self.mapping.get(&msg.channel_pattern) {
                        msg.responder.send(sender.subscribe()).ok();
                    } else {
                        let (sender, receiver) = broadcast::channel(BROADCAST_CAPACITY);
                        self.mapping.insert(msg.channel_pattern.clone(), sender);
                        msg.responder.send(receiver).ok();

                        if let Err(err) = self.pubsub_conn.psubscribe(msg.channel_pattern.as_str()).await {
                            error!(
                                channel_pattern = %msg.channel_pattern,
                                error = %err,
                                "Failed to subscribe to channel pattern"
                            );
                        }
                    }
                }
                // Ugly but otherwise the compiler will complain about needing let bindings
                // Ughh.. Rust, please..
                msg = future::poll_fn(|ctx| self.pubsub_conn.on_message().poll_next_unpin(ctx)) => {
                    if let Some(msg) = msg {
                        let pattern: String = msg.get_pattern().unwrap();

                        if let Some(sender) = self.mapping.get(&pattern) {
                            if sender.send(msg.get_payload_bytes().to_vec()).is_err() {
                                // According to the tokio docs, this case only occurs when all receivers have been dropped
                                // So we can safely delete all the hashmap entry, to keep it as small as possible
                                self.mapping.remove(&pattern);

                                if let Err(err) = self.pubsub_conn.punsubscribe(pattern.as_str()).await {
                                    error!(
                                        channel_pattern = %pattern,
                                        error = %err,
                                        "Failed to unsubscribe from channel pattern"
                                    );
                                }
                            }
                        } else {
                            debug!(%pattern, "Failed to find correct receiver");
                        }
                    } else {
                        // Reconnect, because an ending stream isn't good..
                        self.pubsub_conn = loop {
                            match self.redis_client.get_async_connection().await {
                                Ok(conn) => break conn.into_pubsub(),
                                Err(err) => {
                                    error!(error = %err, "Failed to connect to Redis instance");
                                    tokio::time::sleep(CONNECTION_RETRY_DELAY).await;
                                }
                            }
                        };

                        for key in self.mapping.keys() {
                            if let Err(err) = self.pubsub_conn.psubscribe(key).await {
                                error!(
                                    error = %err,
                                    channel_name = %key,
                                    "Failed to subscribe to channel pattern"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn spawn(
        redis_client: redis::Client,
        pubsub_conn: PubSub,
    ) -> mpsc::Sender<RegistrationMessage> {
        let (sender, receiver) = mpsc::channel(REGISTRATION_QUEUE_SIZE);

        let actor = Self {
            mapping: HashMap::new(),
            pubsub_conn,
            redis_client,
            registration_queue: receiver,
        };
        tokio::spawn(actor.run());

        sender
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
        let sub_connection = client.get_async_connection().await?.into_pubsub();
        let sub_actor = MultiplexActor::spawn(client.clone(), sub_connection);
        let pub_connection = ConnectionManager::new(client).await?;

        Ok(Self {
            pub_connection,
            sub_actor,
        })
    }
}

#[async_trait]
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
    ) -> Result<BoxStream<'static, Result<Vec<u8>>>> {
        let (sender, receiver) = oneshot::channel();
        self.sub_actor
            .send(RegistrationMessage {
                channel_pattern: channel_name,
                responder: sender,
            })
            .await?;
        let broadcast_receiver = receiver.await?;

        Ok(BroadcastStream::new(broadcast_receiver)
            .map_err(Into::into)
            .boxed())
    }
}
