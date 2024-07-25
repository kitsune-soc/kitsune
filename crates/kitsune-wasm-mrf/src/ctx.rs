use crate::{kv_storage, mrf_wit::v1::fep::mrf::keyvalue};
use slab::Slab;
use triomphe::Arc;
use wasmtime::{
    component::{Resource, ResourceTable},
    Engine, Store,
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

pub struct Context {
    pub kv_ctx: KvContext,
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
    storage: Arc<kv_storage::BackendDispatch>,
) -> Store<Context> {
    let wasi_ctx = WasiCtxBuilder::new()
        .allow_ip_name_lookup(false)
        .allow_tcp(false)
        .allow_udp(false)
        .build();

    Store::new(
        engine,
        Context {
            kv_ctx: KvContext {
                module_name: None,
                storage,
                buckets: Slab::new(),
            },
            resource_table: ResourceTable::new(),
            wasi_ctx,
        },
    )
}
