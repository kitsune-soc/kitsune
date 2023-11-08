#![doc = include_str!("../README.md")]
#![forbid(missing_docs, rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    forbidden_lint_groups
)]

#[macro_use]
extern crate tracing;

use enum_dispatch::enum_dispatch;
use futures_util::{stream::BoxStream, Stream};
use pin_project_lite::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    error::Error,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{self, ready, Poll},
};

/// Boxed error
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Type alias for Result, defaulting to [`BoxError`] on the error branch
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

mod util;

pub mod redis;
pub mod tokio_broadcast;

/// Enum dispatch over all supported backends
#[enum_dispatch(MessagingBackend)]
pub enum AnyMessagingBackend {
    /// Redis backend
    Redis(redis::RedisMessagingBackend),

    /// Tokio broadcast backend
    Tokio(tokio_broadcast::TokioBroadcastMessagingBackend),
}

/// Messaging backend
///
/// This is the trait that lets the message hub create emitters and consumers.
/// The backend just needs to be able to transport bytes, that's all.
///
/// The trait is designed to be object-safe since it's internally stored inside an `Arc`
/// and supposed to be type-erased for ease of testing.
#[enum_dispatch]
#[allow(async_fn_in_trait)] // Because of `enum_dispatch`
pub trait MessagingBackend {
    /// Enqueue a new message onto the backend
    async fn enqueue(&self, channel_name: &str, message: Vec<u8>) -> Result<()>;

    /// Open a new stream of messages from the backend
    async fn message_stream(
        &self,
        channel_name: String,
    ) -> Result<BoxStream<'static, Result<Vec<u8>>>>;
}

pin_project! {
    /// Consumer of messages
    pub struct MessageConsumer<M> {
        backend: Arc<AnyMessagingBackend>,
        channel_name: String,
        #[pin]
        inner: BoxStream<'static, Result<Vec<u8>>>,
        _ty: PhantomData<M>,
    }
}

impl<M> MessageConsumer<M>
where
    M: DeserializeOwned + Serialize,
{
    /// Duplicate the message consumer
    ///
    /// This is essentially just creating another consumer.
    /// Useful if you don't have access to the backend nor an emitter
    ///
    /// # Errors
    ///
    /// - Failed to create another consumer
    ///
    /// For more details, check [`MessagingHub::consumer`]
    pub async fn duplicate(&self) -> Result<Self> {
        MessagingHub {
            backend: self.backend.clone(),
        }
        .consumer(self.channel_name.clone())
        .await
    }

    /// Create an emitter that emits messages to this consumer
    #[must_use]
    pub fn emitter(&self) -> MessageEmitter<M> {
        MessagingHub {
            backend: self.backend.clone(),
        }
        .emitter(self.channel_name.clone())
    }

    /// Reconnect the message consumer
    ///
    /// Use this if the stream ever ends and you think it really shouldn't
    ///
    /// # Errors
    ///
    /// - Reconnection failed
    pub async fn reconnect(&mut self) -> Result<()> {
        self.inner = self
            .backend
            .message_stream(self.channel_name.clone())
            .await?;

        Ok(())
    }
}

impl<M> Stream for MessageConsumer<M>
where
    M: DeserializeOwned,
{
    type Item = Result<M>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match ready!(this.inner.poll_next(cx)) {
            Some(Ok(mut msg)) => {
                Poll::Ready(Some(simd_json::from_slice(&mut msg).map_err(Into::into)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}

/// Message emitter
///
/// This is cheaply clonable. Interally it is a string for the channel name and an `Arc` referencing the backend.
#[derive(Clone)]
pub struct MessageEmitter<M> {
    backend: Arc<AnyMessagingBackend>,
    channel_name: String,
    _ty: PhantomData<M>,
}

impl<M> MessageEmitter<M>
where
    M: DeserializeOwned + Serialize,
{
    /// Create a new consumer from the emitter
    ///
    /// # Errors
    ///
    /// - Failed to create consumer
    pub async fn consumer(&self) -> Result<MessageConsumer<M>> {
        MessagingHub {
            backend: self.backend.clone(),
        }
        .consumer(self.channel_name.clone())
        .await
    }

    /// Emit a new message
    ///
    /// # Errors
    ///
    /// - Message failed to serialise
    /// - Message failed to enqueue
    pub async fn emit(&self, message: M) -> Result<()> {
        let message = simd_json::to_vec(&message)?;
        self.backend.enqueue(&self.channel_name, message).await
    }
}

/// Central hub for messaging
///
/// Allows for the registration of new emitters and consumers
///
/// Using the same backend instance ensures that channels with the same name are connected.
/// When using two distinct backend instances it depends on the backend.
///
/// For example, the Redis backend, when connected to the same Redis server, will connect channels with the same name across two different instances.
pub struct MessagingHub {
    backend: Arc<AnyMessagingBackend>,
}

impl MessagingHub {
    /// Create a new messaging hub
    pub fn new<B>(backend: B) -> Self
    where
        B: Into<AnyMessagingBackend>,
    {
        Self {
            backend: Arc::new(backend.into()),
        }
    }

    /// Create a new consumer of messages emitted to the channel
    ///
    /// # Errors
    ///
    /// - Consumer failed to be created
    pub async fn consumer<M>(&self, channel_name: String) -> Result<MessageConsumer<M>>
    where
        M: DeserializeOwned + Serialize,
    {
        let message_stream = self.backend.message_stream(channel_name.clone()).await?;

        Ok(MessageConsumer {
            backend: self.backend.clone(),
            channel_name,
            inner: message_stream,
            _ty: PhantomData,
        })
    }

    /// Create a new emitter for a channel
    #[must_use]
    pub fn emitter<M>(&self, channel_name: String) -> MessageEmitter<M>
    where
        M: DeserializeOwned + Serialize,
    {
        MessageEmitter {
            channel_name,
            backend: self.backend.clone(),
            _ty: PhantomData,
        }
    }
}
