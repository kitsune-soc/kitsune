use crate::{
    kv_storage,
    mrf_wit::v1::fep::mrf::{http_client, keyvalue},
};
use slab::Slab;
use triomphe::Arc;
use wasmtime::{
    Engine, Store, StoreLimits, StoreLimitsBuilder,
    component::{Resource, ResourceTable},
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

pub struct KvContext {
    pub module_name: Option<String>,
    pub storage: Arc<kv_storage::BackendDispatch>,
    pub buckets: Slab<kv_storage::BucketBackendDispatch>,
}

impl KvContext {
    #[inline]
    pub fn get_bucket(
        &self,
        rep: &Resource<keyvalue::Bucket>,
    ) -> &kv_storage::BucketBackendDispatch {
        &self.buckets[rep.rep() as usize]
    }
}

pub struct HttpContext {
    pub client: kitsune_http_client::Client,
    pub bodies: Slab<crate::http_client::Body>,
}

impl HttpContext {
    #[inline]
    pub fn get_body(
        &mut self,
        rep: &Resource<http_client::ResponseBody>,
    ) -> &mut crate::http_client::Body {
        &mut self.bodies[rep.rep() as usize]
    }
}

pub struct Context {
    pub http_ctx: HttpContext,
    pub kv_ctx: KvContext,
    pub resource_limiter: StoreLimits,
    pub resource_table: ResourceTable,
    pub wasi_ctx: WasiCtx,
}

impl WasiView for Context {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

#[inline]
pub fn construct_store(
    engine: &Engine,
    http_client: kitsune_http_client::Client,
    storage: Arc<kv_storage::BackendDispatch>,
) -> Store<Context> {
    let wasi_ctx = WasiCtxBuilder::new()
        .allow_ip_name_lookup(false)
        .allow_tcp(false)
        .allow_udp(false)
        .build();

    let data = Context {
        http_ctx: HttpContext {
            client: http_client,
            bodies: Slab::new(),
        },
        kv_ctx: KvContext {
            module_name: None,
            storage,
            buckets: Slab::new(),
        },
        resource_limiter: StoreLimitsBuilder::new()
            .memory_size(15 * 1024 * 1024)
            .build(),
        resource_table: ResourceTable::new(),
        wasi_ctx,
    };

    let mut store = Store::new(engine, data);
    store.limiter(|store| &mut store.resource_limiter);
    store
}
