//!
//! Messaging abstraction for Kitsune
//!

#![forbid(missing_docs, rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::doc_markdown,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

#[macro_use]
extern crate tracing;

use async_trait::async_trait;
use futures_util::{stream::BoxStream, Stream, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, future, marker::PhantomData, sync::Arc};

/// Boxed error
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Type alias for Result, defaulting to [`BoxError`] on the error branch
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

pub mod redis;

/// Messaging backend
///
/// This is the trait that lets the message hub create emitters and consumers.
/// The backend just needs to be able to transport bytes, that's all.
///
/// The trait is designed to be object-safe since it's internally stored inside an `Arc`
/// and supposed to be type-erased for ease of testing.
#[async_trait]
pub trait MessagingBackend {
    /// Enqueue a new message onto the backend
    async fn enqueue(&self, channel_name: &str, message: Vec<u8>) -> Result<()>;

    /// Open a new stream of messages from the backend
    async fn message_stream(
        &self,
        channel_name: String,
    ) -> Result<BoxStream<'static, Result<Vec<u8>>>>;
}

/// Message emitter
///
/// This is cheaply clonable. Interally it is a string for the channel name and an `Arc` referencing the backend.
#[derive(Clone)]
pub struct MessageEmitter<M> {
    backend: Arc<dyn MessagingBackend + Send + Sync>,
    channel_name: String,
    _ty: PhantomData<M>,
}

impl<M> MessageEmitter<M>
where
    M: Serialize,
{
    /// Emit a new message
    ///
    /// # Errors
    ///
    /// - Message failed to serialise
    /// - Message failed to enqueue
    pub async fn emit(&self, message: M) -> Result<()> {
        let message = serde_json::to_vec(&message)?;
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
    backend: Arc<dyn MessagingBackend + Send + Sync>,
}

impl MessagingHub {
    /// Create a new messaging hub
    pub fn new<B>(backend: B) -> Self
    where
        B: MessagingBackend + Send + Sync + 'static,
    {
        Self {
            backend: Arc::new(backend),
        }
    }

    /// Create a new consumer of messages emitted to the channel
    ///
    /// # Errors
    ///
    /// - Consumer failed to be created
    pub async fn consumer<M>(&self, channel_name: String) -> Result<impl Stream<Item = Result<M>>>
    where
        M: DeserializeOwned + Send + 'static,
    {
        Ok(self
            .backend
            .message_stream(channel_name)
            .await?
            .and_then(|msg| future::ready(serde_json::from_slice(&msg).map_err(BoxError::from))))
    }

    /// Create a new emitter for a channel
    #[must_use]
    pub fn emitter<M>(&self, channel_name: String) -> MessageEmitter<M> {
        MessageEmitter {
            channel_name,
            backend: self.backend.clone(),
            _ty: PhantomData,
        }
    }
}
