use futures_util::StreamExt;
use kitsune_messaging::{redis::RedisMessagingBackend, MessagingHub};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let redis_backend = RedisMessagingBackend::new("redis://localhost")
        .await
        .unwrap();
    let hub = MessagingHub::new(redis_backend);

    let emitter = hub.emitter("test".into());
    let consumer = hub.consumer::<Value>("test".into()).await.unwrap();

    emitter
        .emit(json!({
            "hello": "world",
            "who": ["are", "you", "?"]
        }))
        .await
        .unwrap();

    consumer
        .for_each(|msg| async move { println!("{:#?}]", msg) })
        .await;
}
