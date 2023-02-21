#[macro_use]
extern crate tracing;

use async_trait::async_trait;
use futures_util::stream::BoxStream;
use std::error::Error;

pub type BoxError = Box<dyn Error + Send + Sync>;
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

pub mod redis;

#[async_trait]
pub trait MessagingBackend {
    async fn enqueue(&mut self, channel_name: String, message: Vec<u8>) -> Result<()>;
    async fn event_stream(
        &self,
        channel_name: String,
    ) -> Result<BoxStream<'static, Result<Vec<u8>>>>;
}
