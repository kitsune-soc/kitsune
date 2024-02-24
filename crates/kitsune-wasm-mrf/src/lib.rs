#[macro_use]
extern crate tracing;

use futures_util::{stream::FuturesUnordered, TryStreamExt};
use miette::IntoDiagnostic;
use std::{fmt::Debug, path::Path, sync::Arc};
use tokio::fs;
use typed_builder::TypedBuilder;
use walkdir::WalkDir;
use wasmtime::{component::Component, Config, Engine, InstanceAllocationStrategy};

mod mrf_wit {
    wasmtime::component::bindgen!();
}

#[derive(Clone, TypedBuilder)]
pub struct MrfService {
    components: Arc<[Component]>,
    engine: Engine,
}

impl MrfService {
    #[instrument]
    pub async fn from_directory<P>(dir: P) -> miette::Result<Self>
    where
        P: AsRef<Path> + Debug,
    {
        let mut config = Config::new();
        config
            .allocation_strategy(InstanceAllocationStrategy::pooling())
            .async_support(true)
            .epoch_interruption(true)
            .wasm_component_model(true);
        let engine = Engine::new(&config).map_err(miette::Report::msg)?;

        // Read all the `.wasm` files from the disk
        // Recursively traverse the entire directory tree doing so and follow all symlinks
        // Also run the I/O operations inside a `FuturesUnordered` to enable concurrent reading
        let mut wasm_data_stream = WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                (entry.path().is_file() && entry.path().ends_with(".wasm"))
                    .then(|| entry.into_path())
            })
            .inspect(|path| debug!(?path, "discovered WASM module"))
            .map(fs::read)
            .collect::<FuturesUnordered<_>>();

        let mut components = Vec::new();
        while let Some(wasm_data) = wasm_data_stream.try_next().await.into_diagnostic()? {
            components.push(Component::new(&engine, wasm_data).map_err(miette::Report::msg)?);
        }

        Ok(Self {
            components: components.into(),
            engine,
        })
    }
}
