use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    ManifestParse(#[from] mrf_manifest::ParseError),

    #[error(transparent)]
    Runtime(wasmtime::Error),
}