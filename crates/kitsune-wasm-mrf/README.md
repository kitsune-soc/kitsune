# kitsune-wasm-mrf

Kitsune's implementation of the FEP draft for WASM-based MRFs

## Note on the WASM binary inside the `tests/` directory

The binary is used to verify whether the library can run MRF modules.  
To reproduce the binary (or rather the function of the binary, codegen might differ), compile the `example-mrf` project like so:

- Download `wasi_snapshot_preview1.reactor.wasm` from the latest release page of wasmtime
- Install `wasm-tools`
- Compile the project with `cargo build --target wasm32-wasi --profile=dist`
- Link the snapshot to the build artifact with `wasm-tools component new example_mrf.wasm -o example_mrf.component.wasm --adapt wasi_snapshot_preview1.reactor.wasm`
