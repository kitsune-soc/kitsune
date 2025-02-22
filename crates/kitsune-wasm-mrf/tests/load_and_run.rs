use kitsune_wasm_mrf::{MrfModule, MrfService, Outcome};
use smol_str::SmolStr;
use std::borrow::Cow;
use tempfile::NamedTempFile;
use wasmtime::{Config, Engine, component::Component};

const WASM_COMPONENT: &[u8] = include_bytes!("example_mrf.component.wasm");

#[tokio::test]
async fn basic() {
    tracing_subscriber::fmt::init();

    let db_file = NamedTempFile::new().unwrap();
    let fs_backend = kitsune_wasm_mrf::kv_storage::FsBackend::from_path(db_file.path()).unwrap();

    let mut config = Config::new();
    config.async_support(true).wasm_component_model(true);
    let engine = Engine::new(&config).unwrap();
    let component = Component::new(&engine, WASM_COMPONENT).unwrap();

    let (manifest, ..) = mrf_manifest::decode(WASM_COMPONENT).unwrap().unwrap();
    let mrf_manifest::Manifest::V1(manifest) = manifest else {
        panic!();
    };

    let service = MrfService::from_components(
        engine,
        vec![MrfModule {
            component,
            config: SmolStr::default(),
            manifest,
        }],
        kitsune_http_client::Client::default(),
        fs_backend.into(),
    )
    .unwrap();
    let result = service
        .handle_incoming("[anything]", "[imagine activity here]")
        .await
        .unwrap();

    assert_eq!(
        result,
        Outcome::Accept(Cow::Borrowed("[imagine activity here]"))
    );
}
