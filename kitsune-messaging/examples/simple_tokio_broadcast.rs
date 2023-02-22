use futures_util::StreamExt;
use kitsune_messaging::{tokio_broadcast::TokioBroadcastMessagingBackend, MessagingHub};
use serde_json::{json, Value};
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let tokio_broadcast_backend = TokioBroadcastMessagingBackend::default();
    let hub = MessagingHub::new(tokio_broadcast_backend);

    let consumer = hub.consumer::<Value>("test".into()).await.unwrap();
    tokio::spawn(consumer.for_each(|msg| async move { println!("Consumer 1: {:?}]", msg) }));

    let consumer = hub.consumer::<Value>("test".into()).await.unwrap();
    tokio::spawn(consumer.for_each(|msg| async move { println!("Consumer 2: {:?}]", msg) }));

    let consumer = hub.consumer::<Value>("test2".into()).await.unwrap();
    tokio::spawn(consumer.for_each(|msg| async move { println!("Consumer 3: {:?}]", msg) }));

    let consumer = hub.consumer::<Value>("test2".into()).await.unwrap();
    tokio::spawn(consumer.for_each(|msg| async move { println!("Consumer 4: {:?}]", msg) }));

    for i in 1..=3 {
        let emitter = hub.emitter("test".into());
        emitter
            .emit(json!({
                "hello": "world",
                "who": ["are", "you", "?"],
                "message": i,
            }))
            .await
            .unwrap();

        let emitter = hub.emitter("test2".into());
        emitter
            .emit(json!({
                "hello": "world",
                "who": ["are", "you", "?"],
                "message": i,
            }))
            .await
            .unwrap();
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
}
