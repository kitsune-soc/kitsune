use kitsune_wasm_mrf::{MrfService, Outcome};
use std::borrow::Cow;
use wasmtime::{component::Component, Config, Engine};

const WASM_COMPONENT: &[u8] = include_bytes!("example_mrf.component.wasm");

#[tokio::test]
async fn basic() {
    let mut config = Config::new();
    config.async_support(true).wasm_component_model(true);
    let engine = Engine::new(&config).unwrap();
    let component = Component::new(&engine, WASM_COMPONENT).unwrap();

    let service = MrfService::from_components(engine, vec![component]).unwrap();
    let result = service
        .handle_incoming("[imagine activity here]")
        .await
        .unwrap();

    assert_eq!(
        result,
        Outcome::Accept(Cow::Borrowed("[imagine activity here]"))
    );
}
