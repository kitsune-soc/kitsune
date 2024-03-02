#[macro_use]
extern crate tracing;

use self::{
    ctx::{construct_store, Context},
    mrf_wit::v1::fep::mrf::types::{Direction, Error as MrfError},
};
use futures_util::{stream::FuturesUnordered, Stream, StreamExt, TryFutureExt, TryStreamExt};
use miette::{Diagnostic, IntoDiagnostic};
use mrf_manifest::Manifest;
use std::{
    borrow::Cow,
    fmt::Debug,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;
use tokio::fs;
use typed_builder::TypedBuilder;
use walkdir::WalkDir;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, InstanceAllocationStrategy,
};

pub use self::error::Error;

mod ctx;
mod error;
mod mrf_wit;

#[inline]
fn find_mrf_modules<P>(dir: P) -> impl Stream<Item = Result<(PathBuf, Vec<u8>), io::Error>>
where
    P: AsRef<Path>,
{
    // Read all the `.wasm` files from the disk
    // Recursively traverse the entire directory tree doing so and follow all symlinks
    // Also run the I/O operations inside a `FuturesUnordered` to enable concurrent reading
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            (entry.path().is_file() && entry.path().extension() == Some("wasm".as_ref()))
                .then(|| entry.into_path())
        })
        .inspect(|path| debug!(?path, "discovered WASM module"))
        .map(|path| fs::read(path.clone()).map_ok(|data| (path, data)))
        .collect::<FuturesUnordered<_>>()
}

#[inline]
fn load_mrf_module(
    engine: &Engine,
    module_path: &Path,
    bytes: &[u8],
) -> miette::Result<Option<(Manifest<'static>, Component)>> {
    let component = Component::new(engine, bytes).map_err(|err| {
        miette::Report::new(ComponentParseError {
            path_help: format!("path to the module: {}", module_path.display()),
            advice: "Did you make the WASM file a component via `wasm-tools`?",
        })
        .wrap_err(err)
    })?;

    let Some((manifest, _section_range)) = mrf_manifest::decode(bytes)? else {
        error!("missing manifest. skipping load.");
        return Ok(None);
    };
    let Manifest::V1(ref manifest_v1) = manifest else {
        error!("invalid manifest version. expected v1");
        return Ok(None);
    };

    info!(name = %manifest_v1.name, version = %manifest_v1.version, "loaded MRF module");

    Ok(Some((manifest.to_owned(), component)))
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

impl MrfService {
    #[inline]
    pub fn from_components(engine: Engine, components: Vec<Component>) -> miette::Result<Self> {
        let mut linker = Linker::<Context>::new(&engine);
        mrf_wit::v1::Mrf::add_to_linker(&mut linker, |ctx| &mut ctx.unit)
            .map_err(miette::Report::msg)?;
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker)
            .map_err(miette::Report::msg)?;

        Ok(Self {
            components: components.into(),
            engine,
            linker: Arc::new(linker),
        })
    }

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
        let wasm_data_stream = find_mrf_modules(dir)
            .map(IntoDiagnostic::into_diagnostic)
            .and_then(|(module_path, wasm_data)| {
                let engine = &engine;

                async move { load_mrf_module(engine, &module_path, &wasm_data) }
            });
        tokio::pin!(wasm_data_stream);

        let mut components = Vec::new();
        while let Some((_manifest, component)) = wasm_data_stream.try_next().await?.flatten() {
            // TODO: Manifest validation, permission grants, etc.
            components.push(component);
        }

        Self::from_components(engine, components)
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
            let (mrf, _) = mrf_wit::v1::Mrf::instantiate_async(&mut store, component, &self.linker)
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

    #[inline]
    pub async fn handle_incoming<'a>(&self, activity: &'a str) -> Result<Outcome<'a>, Error> {
        self.handle(Direction::Incoming, activity).await
    }

    #[inline]
    pub async fn handle_outgoing<'a>(&self, activity: &'a str) -> Result<Outcome<'a>, Error> {
        self.handle(Direction::Outgoing, activity).await
    }
}
