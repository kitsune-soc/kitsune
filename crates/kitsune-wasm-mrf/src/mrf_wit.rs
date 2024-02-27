pub mod transform {
    wasmtime::component::bindgen!({
        async: true,
        world: "mrf-v1",
    });
}

impl transform::fep::mrf::types::Host for () {}
