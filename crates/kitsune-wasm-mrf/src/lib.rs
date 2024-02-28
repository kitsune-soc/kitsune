#[macro_use]
extern crate tracing;

use self::mrf_wit::transform::fep::mrf::types::{Direction, Error as MrfError};
use futures_util::{stream::FuturesUnordered, TryFutureExt, TryStreamExt};
use miette::{Diagnostic, IntoDiagnostic};
use mrf_manifest::Manifest;
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

pub use self::error::Error;

mod error;
mod mrf_wit;

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
#[error("{path_help}")]
struct ComponentParseError {
    path_help: String,
    #[help]
    advice: &'static str,
}

#[derive(Clone, TypedBuilder)]
pub struct MrfService {
    components: Arc<[Component]>,
    engine: Engine,
    linker: Arc<Linker<Context>>,
}

#[inline]
fn construct_store(engine: &Engine) -> Store<Context> {
    let wasi_ctx = WasiCtxBuilder::new()
        .allow_ip_name_lookup(false)
        .allow_tcp(false)
        .allow_udp(false)
        .build();

    Store::new(
        engine,
        Context {
            resource_table: ResourceTable::new(),
            wasi_ctx,
            unit: (),
        },
    )
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
            .map(|path| fs::read(path.clone()).map_ok(|data| (path, data)))
            .collect::<FuturesUnordered<_>>();

        let mut linker = Linker::<Context>::new(&engine);
        mrf_wit::transform::MrfV1::add_to_linker(&mut linker, |ctx| &mut ctx.unit)
            .map_err(miette::Report::msg)?;
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker)
            .map_err(miette::Report::msg)?;

        let mut components = Vec::new();
        while let Some((module_path, wasm_data)) =
            wasm_data_stream.try_next().await.into_diagnostic()?
        {
            let component = Component::new(&engine, &wasm_data).map_err(|err| {
                miette::Report::new(ComponentParseError {
                    path_help: format!("path to the module: {}", module_path.display()),
                    advice: "Did you make the WASM file a component via `wasm-tools`?",
                })
                .wrap_err(err)
            })?;

            let Some((Manifest::V1(manifest), _section_range)) = mrf_manifest::parse(&wasm_data)?
            else {
                error!("missing manifest. skipping load.");
                continue;
            };
            info!(name = %manifest.name, version = %manifest.version, "loading MRF module");

            // TODO: Manifest validation, permission grants, etc.

            components.push(component);
        }

        Ok(Self {
            components: components.into(),
            engine,
            linker: Arc::new(linker),
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
        let mut store = construct_store(&self.engine);
        let mut activity = Cow::Borrowed(activity);

        for component in self.components.iter() {
            let (mrf, _) =
                mrf_wit::transform::MrfV1::instantiate_async(&mut store, component, &self.linker)
                    .await
                    .map_err(Error::Runtime)?;

            // TODO: Load configuration
            let config = "";

            let result = mrf
                .call_transform(&mut store, config, direction, &activity)
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
