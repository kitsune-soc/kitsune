use kitsune_wasm_mrf::{MrfModule, MrfService, Outcome};
use mrf_manifest::{ActivitySet, ApiVersion, Manifest, ManifestV1};
use smol_str::SmolStr;
use std::{borrow::Cow, collections::HashSet};
use wasmtime::{component::Component, Config, Engine};

const WASM_COMPONENT: &[u8] = include_bytes!("example_mrf.component.wasm");

fn dummy_manifest() -> Manifest<'static> {
    Manifest::V1(ManifestV1 {
        api_version: ApiVersion::V1,
        name: "dummy".into(),
        version: "1.0.0".parse().unwrap(),
        activity_types: ActivitySet::from(HashSet::default()),
        config_schema: None,
    })
}

#[tokio::test]
async fn basic() {
    let mut config = Config::new();
    config.async_support(true).wasm_component_model(true);
    let engine = Engine::new(&config).unwrap();
    let component = Component::new(&engine, WASM_COMPONENT).unwrap();

    let service = MrfService::from_components(
        engine,
        vec![MrfModule {
            component,
            config: SmolStr::default(),
            manifest: dummy_manifest(),
        }],
    )
    .unwrap();
    let result = service
        .handle_incoming("[imagine activity here]")
        .await
        .unwrap();

    assert_eq!(
        result,
        Outcome::Accept(Cow::Borrowed("[imagine activity here]"))
    );
}
