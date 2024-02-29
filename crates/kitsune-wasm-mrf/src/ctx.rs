use wasmtime::{component::ResourceTable, Engine, Store};
use wasmtime_wasi::preview2::{WasiCtx, WasiCtxBuilder, WasiView};

pub struct Context {
    pub resource_table: ResourceTable,
    pub wasi_ctx: WasiCtx,
    pub unit: (),
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
pub fn construct_store(engine: &Engine) -> Store<Context> {
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
