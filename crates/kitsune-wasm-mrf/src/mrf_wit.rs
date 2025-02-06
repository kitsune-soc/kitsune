pub mod v1 {
    wasmtime::component::bindgen!({
        async: true,
        tracing: true,
        world: "mrf",
    });
}

impl v1::fep::mrf::types::Host for crate::ctx::Context {}
