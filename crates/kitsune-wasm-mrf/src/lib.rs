#[macro_use]
extern crate tracing;

use self::mrf_wit::fep::mrf::types::Error as MrfError;
use futures_util::{stream::FuturesUnordered, TryStreamExt};
use miette::{Diagnostic, IntoDiagnostic};
use mrf_wit::fep::mrf::types::Direction;
use std::{borrow::Cow, fmt::Debug, path::Path, sync::Arc};
use thiserror::Error;
use tokio::fs;
use typed_builder::TypedBuilder;
use walkdir::WalkDir;
use wasmtime::{
    component::{Component, Linker, ResourceTable},
    Config, Engine, InstanceAllocationStrategy, Store,
};
use wasmtime_wasi::preview2::{WasiCtx, WasiCtxBuilder, WasiView};

mod mrf_wit {
    wasmtime::component::bindgen!({ async: true });

    impl fep::mrf::types::Host for () {}
}

struct Context {
    resource_table: ResourceTable,
    wasi_ctx: WasiCtx,
    unit: (),
}

impl WasiView for Context {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Outcome<'a> {
    Accept(Cow<'a, str>),
    Reject,
}

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    Runtime(wasmtime::Error),
}

#[derive(Clone, TypedBuilder)]
pub struct MrfService {
    components: Arc<[mrf_wit::Mrf]>,
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
                (entry.path().is_file() && entry.path().extension() == Some("wasm".as_ref()))
                    .then(|| entry.into_path())
            })
            .inspect(|path| debug!(?path, "discovered WASM module"))
            .map(fs::read)
            .collect::<FuturesUnordered<_>>();

        let wasi_ctx = WasiCtxBuilder::new()
            .allow_ip_name_lookup(true)
            .allow_tcp(true)
            .allow_udp(true)
            .inherit_network()
            .build();

        let mut store = Store::new(
            &engine,
            Context {
                resource_table: ResourceTable::new(),
                wasi_ctx,
                unit: (),
            },
        );

        let mut linker = Linker::<Context>::new(&engine);
        mrf_wit::Mrf::add_to_linker(&mut linker, |ctx| &mut ctx.unit)
            .map_err(miette::Report::msg)?;
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker)
            .map_err(miette::Report::msg)?;

        let mut components = Vec::new();
        while let Some(wasm_data) = wasm_data_stream.try_next().await.into_diagnostic()? {
            let component = Component::new(&engine, wasm_data).map_err(miette::Report::msg)?;
            let (bindings, _) = mrf_wit::Mrf::instantiate_async(&mut store, &component, &linker)
                .await
                .map_err(miette::Report::msg)?;

            let meta = bindings.fep_mrf_meta();
            let module_name = meta
                .call_name(&mut store)
                .await
                .map_err(miette::Report::msg)?;
            let module_version = meta
                .call_version(&mut store)
                .await
                .map_err(miette::Report::msg)?;

            info!(name = %module_name, version = %module_version, "loaded MRF module");

            components.push(bindings);
        }

        Ok(Self {
            components: components.into(),
            engine,
        })
    }

    #[must_use]
    pub fn module_count(&self) -> usize {
        self.components.len()
    }

    async fn handle<'a>(
        &self,
        direction: Direction,
        activity: &'a str,
    ) -> Result<Outcome<'a>, Error> {
        let mut store = Store::new(&self.engine, ());
        let mut activity = Cow::Borrowed(activity);

        for mrf in self.components.iter() {
            let result = mrf
                .fep_mrf_transform()
                .call_transform(&mut store, direction, &activity)
                .await
                .map_err(Error::Runtime)?;

            match result {
                Ok(transformed) => {
                    activity = Cow::Owned(transformed);
                }
                Err(MrfError::ErrorContinue(msg)) => {
                    error!(%msg, "MRF errored out. Continuing.");
                }
                Err(MrfError::ErrorReject(msg)) => {
                    error!(%msg, "MRF errored out. Aborting.");
                    return Ok(Outcome::Reject);
                }
                Err(MrfError::Reject) => {
                    error!("MRF rejected activity. Aborting.");
                    return Ok(Outcome::Reject);
                }
            }
        }

        Ok(Outcome::Accept(activity))
    }

    pub async fn handle_incoming<'a>(&self, activity: &'a str) -> Result<Outcome<'a>, Error> {
        self.handle(Direction::Incoming, activity).await
    }

    pub async fn handle_outgoing<'a>(&self, activity: &'a str) -> Result<Outcome<'a>, Error> {
        self.handle(Direction::Outgoing, activity).await
    }
}
