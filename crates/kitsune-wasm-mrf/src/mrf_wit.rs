pub mod v1 {
    wasmtime::component::bindgen!({
        async: true,
        world: "mrf",
    });
}

impl v1::fep::mrf::types::Host for crate::ctx::Context {}
