//!
//! Implementation over a Tokio broadcast channel
//!

use crate::{MessagingBackend, Result};
use futures_util::{Stream, StreamExt, TryStreamExt};
use std::{collections::HashMap, sync::RwLock};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

const BROADCAST_CAPACITY: usize = 50;

/// Messaging backend implementation based on Tokio's broadcast channel
pub struct TokioBroadcastMessagingBackend {
    registry: RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>,
}

impl Default for TokioBroadcastMessagingBackend {
    fn default() -> Self {
        Self {
            registry: RwLock::new(HashMap::new()),
        }
    }
}

impl MessagingBackend for TokioBroadcastMessagingBackend {
    async fn enqueue(&self, channel_name: &str, message: Vec<u8>) -> Result<()> {
        let guard = self.registry.read().unwrap();
        if let Some(sender) = guard.get(channel_name) {
            sender.send(message)?;
        }

        Ok(())
    }

    async fn message_stream(
        &self,
        channel_name: String,
    ) -> Result<impl Stream<Item = Result<Vec<u8>>> + 'static> {
        let guard = self.registry.read().unwrap();
        let receiver = if let Some(sender) = guard.get(&channel_name) {
            sender.subscribe()
        } else {
            drop(guard);

            let mut guard = self.registry.write().unwrap();
            let (sender, receiver) = broadcast::channel(BROADCAST_CAPACITY);
            guard.insert(channel_name, sender);
            receiver
        };

        Ok(BroadcastStream::new(receiver).map_err(Into::into).boxed())
    }
}
