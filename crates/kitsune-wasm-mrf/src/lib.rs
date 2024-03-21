#[macro_use]
extern crate tracing;

use self::{
    ctx::{construct_store, Context},
    mrf_wit::v1::fep::mrf::types::{Direction, Error as MrfError},
};
use futures_util::{stream::FuturesUnordered, Stream, StreamExt, TryFutureExt, TryStreamExt};
use kitsune_config::mrf::{
    Configuration as MrfConfiguration, FsKvStorage, KvStorage, RedisKvStorage,
};
use kitsune_type::ap::Activity;
use miette::{Diagnostic, IntoDiagnostic};
use mrf_manifest::{Manifest, ManifestV1};
use smol_str::SmolStr;
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
mod logging;
mod mrf_wit;

pub mod kv_storage;

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
) -> miette::Result<Option<(ManifestV1<'static>, Component)>> {
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

    Ok(Some((manifest_v1.to_owned(), component)))
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

pub struct MrfModule {
    pub component: Component,
    pub config: SmolStr,
    pub manifest: ManifestV1<'static>,
}

#[derive(Clone, TypedBuilder)]
pub struct MrfService {
    engine: Engine,
    linker: Arc<Linker<Context>>,
    modules: Arc<[MrfModule]>,
    storage: Arc<kv_storage::BackendDispatch>,
}

impl MrfService {
    #[inline]
    pub fn from_components(
        engine: Engine,
        modules: Vec<MrfModule>,
        storage: kv_storage::BackendDispatch,
    ) -> miette::Result<Self> {
        let mut linker = Linker::<Context>::new(&engine);

        mrf_wit::v1::Mrf::add_to_linker(&mut linker, |ctx| ctx).map_err(miette::Report::msg)?;
        wasmtime_wasi::command::add_to_linker(&mut linker).map_err(miette::Report::msg)?;

        Ok(Self {
            engine,
            linker: Arc::new(linker),
            modules: modules.into(),
            storage: Arc::new(storage),
        })
    }

    #[instrument(skip_all, fields(module_dir = %config.module_dir))]
    pub async fn from_config(config: &MrfConfiguration) -> miette::Result<Self> {
        let storage = match config.storage {
            KvStorage::Fs(FsKvStorage { ref path }) => {
                kv_storage::FsBackend::from_path(path.as_str())?.into()
            }
            KvStorage::Redis(RedisKvStorage { ref url, pool_size }) => {
                let client = redis::Client::open(url.as_str()).into_diagnostic()?;
                kv_storage::RedisBackend::from_client(client, pool_size.get())
                    .await?
                    .into()
            }
        };

        let mut engine_config = Config::new();
        engine_config
            .allocation_strategy(InstanceAllocationStrategy::pooling())
            .async_support(true)
            .wasm_component_model(true);

        let engine = Engine::new(&engine_config).map_err(miette::Report::msg)?;
        let wasm_data_stream = find_mrf_modules(config.module_dir.as_str())
            .map(IntoDiagnostic::into_diagnostic)
            .and_then(|(module_path, wasm_data)| {
                let engine = &engine;

                async move { load_mrf_module(engine, &module_path, &wasm_data) }
            });
        tokio::pin!(wasm_data_stream);

        let mut modules = Vec::new();
        while let Some((manifest, component)) = wasm_data_stream.try_next().await?.flatten() {
            // TODO: permission grants, etc.

            let span = info_span!(
                "load_mrf_module_config",
                name = %manifest.name,
                version = %manifest.version,
            );

            let config = span.in_scope(|| {
                config
                    .module_config
                    .get(&*manifest.name)
                    .cloned()
                    .inspect(|_| debug!("found configuration"))
                    .unwrap_or_else(|| {
                        debug!("didn't find configuration. defaulting to empty string");
                        SmolStr::default()
                    })
            });

            let module = MrfModule {
                component,
                config,
                manifest,
            };

            modules.push(module);
        }

        Self::from_components(engine, modules, storage)
    }

    #[must_use]
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    async fn handle<'a>(
        &self,
        direction: Direction,
        activity_type: &str,
        activity: &'a str,
    ) -> Result<Outcome<'a>, Error> {
        let mut store = construct_store(&self.engine, self.storage.clone());
        let mut activity = Cow::Borrowed(activity);

        for module in self.modules.iter() {
            let activity_types = &module.manifest.activity_types;
            if !activity_types.all_activities() && !activity_types.contains(activity_type) {
                continue;
            }

            let (mrf, _) =
                mrf_wit::v1::Mrf::instantiate_async(&mut store, &module.component, &self.linker)
                    .await
                    .map_err(Error::Runtime)?;

            store.data_mut().kv_ctx.module_name = Some(module.manifest.name.to_string());

            let result = mrf
                .call_transform(&mut store, &module.config, direction, &activity)
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
    pub async fn handle_incoming<'a>(
        &self,
        activity_type: &str,
        activity: &'a str,
    ) -> Result<Outcome<'a>, Error> {
        self.handle(Direction::Incoming, activity_type, activity)
            .await
    }

    #[inline]
    pub async fn handle_outgoing(&self, activity: &Activity) -> Result<Outcome<'static>, Error> {
        let serialised = simd_json::to_string(activity)?;
        let outcome = self
            .handle(Direction::Outgoing, activity.r#type.as_ref(), &serialised)
            .await?;

        let outcome: Outcome<'static> = match outcome {
            Outcome::Accept(Cow::Borrowed(..)) => {
                // As per the logic in the previous function, we can assume that if the Cow is owned, it has been modified
                // If it hasn't been modified it is in its borrowed state
                //
                // Therefore we don't need to allocate again here, simply reconstruct a new `Outcome` with an owned Cow.
                Outcome::Accept(Cow::Owned(serialised))
            }
            Outcome::Accept(Cow::Owned(owned)) => Outcome::Accept(Cow::Owned(owned)),
            Outcome::Reject => Outcome::Reject,
        };

        Ok(outcome)
    }
}
