use kitsune_wasm_mrf::{MrfService, Outcome};
use std::borrow::Cow;
use tracing::level_filters::LevelFilter;

#[tokio::test]
async fn basic() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let service = MrfService::from_directory("tests").await.unwrap();
    let result = service
        .handle_incoming("[imagine activity here]")
        .await
        .unwrap();

    assert_eq!(service.module_count(), 1);
    assert_eq!(
        result,
        Outcome::Accept(Cow::Owned("[imagine activity here]".into()))
    );
}
