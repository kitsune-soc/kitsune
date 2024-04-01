use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Json(#[from] simd_json::Error),

    #[error(transparent)]
    ManifestParse(#[from] mrf_manifest::DecodeError),

    #[error(transparent)]
    Runtime(wasmtime::Error),
}
