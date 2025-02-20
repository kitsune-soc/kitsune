use bytes::Bytes;
use http_body_util::Full;
use kitsune_wasm_mrf::{MrfModule, MrfService, Outcome};
use smol_str::SmolStr;
use std::{borrow::Cow, convert::Infallible};
use tempfile::NamedTempFile;
use wasmtime::{Config, Engine, component::Component};

const WASM_COMPONENT: &[u8] = include_bytes!("http_client_test.component.wasm");

async fn handle_request(
    req: http::Request<kitsune_http_client::Body>,
) -> Result<http::Response<Full<Bytes>>, Infallible> {
    assert_eq!(req.uri().host(), Some("aumetra.xyz"));
    assert_eq!(req.uri().path(), "/blog");
    assert_eq!(req.uri().scheme_str(), Some("https"));

    let response = http::Response::builder()
        .status(200)
        .body(Full::new(Bytes::from_static(b"[response here]")))
        .unwrap();

    Ok(response)
}

#[tokio::test]
async fn request() {
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

    let mocked_client =
        kitsune_http_client::Client::builder().service(tower::service_fn(handle_request));
    let service = MrfService::from_components(
        engine,
        vec![MrfModule {
            component,
            config: SmolStr::default(),
            manifest,
        }],
        mocked_client,
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
