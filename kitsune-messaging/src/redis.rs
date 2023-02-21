use crate::{MessagingBackend, Result};
use async_trait::async_trait;
use futures_util::{future, stream::BoxStream, StreamExt, TryStreamExt};
use redis::{
    aio::{Connection, PubSub},
    AsyncCommands, RedisError,
};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;

const BROADCAST_CAPACITY: usize = 10;
const REGISTRATION_QUEUE_SIZE: usize = 50;

#[derive(Debug)]
struct RegistrationMessage {
    channel_pattern: String,
    responder: oneshot::Sender<broadcast::Receiver<Vec<u8>>>,
}

struct MultiplexActor {
    mapping: HashMap<String, broadcast::Sender<Vec<u8>>>,
    pubsub_conn: PubSub,
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
                Some(msg) = future::poll_fn(|ctx| self.pubsub_conn.on_message().poll_next_unpin(ctx)) => {
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
                    }
                }
            }
        }
    }

    pub fn spawn(pubsub_conn: PubSub) -> mpsc::Sender<RegistrationMessage> {
        let (sender, receiver) = mpsc::channel(REGISTRATION_QUEUE_SIZE);

        let actor = Self {
            mapping: HashMap::new(),
            pubsub_conn,
            registration_queue: receiver,
        };
        tokio::spawn(actor.run());

        sender
    }
}

pub struct RedisMessagingBackend {
    pub_connection: Connection,
    sub_actor: mpsc::Sender<RegistrationMessage>,
}

impl RedisMessagingBackend {
    pub async fn new(conn_string: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(conn_string)?;
        let pub_connection = client.get_async_connection().await?;
        let sub_connection = client.get_async_connection().await?.into_pubsub();
        let sub_actor = MultiplexActor::spawn(sub_connection);

        Ok(Self {
            pub_connection,
            sub_actor,
        })
    }
}

#[async_trait]
impl MessagingBackend for RedisMessagingBackend {
    async fn enqueue(&mut self, channel_name: String, message: Vec<u8>) -> Result<()> {
        self.pub_connection
            .publish(channel_name, message)
            .await
            .map_err(Into::into)
    }

    async fn event_stream(
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
