#[macro_use]
extern crate tracing;

use async_trait::async_trait;
use futures_util::{stream::BoxStream, Stream, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, future, marker::PhantomData, sync::Arc};

pub type BoxError = Box<dyn Error + Send + Sync>;
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

pub mod redis;

#[async_trait]
pub trait MessagingBackend {
    async fn enqueue(&self, channel_name: &str, message: Vec<u8>) -> Result<()>;
    async fn message_stream(
        &self,
        channel_name: String,
    ) -> Result<BoxStream<'static, Result<Vec<u8>>>>;
}

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
    pub async fn emit(&self, message: M) -> Result<()> {
        let message = serde_json::to_vec(&message)?;
        self.backend.enqueue(&self.channel_name, message).await
    }
}

pub struct MessagingThing {
    backend: Arc<dyn MessagingBackend + Send + Sync>,
}

impl MessagingThing {
    pub fn new<B>(backend: B) -> Self
    where
        B: MessagingBackend + Send + Sync + 'static,
    {
        Self {
            backend: Arc::new(backend),
        }
    }

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

    pub async fn emitter<M>(&self, channel_name: String) -> MessageEmitter<M> {
        MessageEmitter {
            channel_name,
            backend: self.backend.clone(),
            _ty: PhantomData,
        }
    }
}
