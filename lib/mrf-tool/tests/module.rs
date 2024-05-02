use mrf_tool::DummyFs;
use std::io;

#[test]
fn validate() {
    let mut fs = DummyFs::default();
    fs.insert("empty.wasm".into(), wat::parse_str("(module)").unwrap());
    fs.insert("empty.bin".into(), vec![0x00]);

    let result = mrf_tool::handle(
        &mut fs,
        &mut io::sink(),
        ["mrf-tool", "module", "validate", "empty.wasm"],
    );
    assert!(result.is_ok(), "{result:?}");

    let error = mrf_tool::handle(
        &mut fs,
        &mut io::sink(),
        ["mrf-tool", "module", "validate", "empty.bin"],
    )
    .unwrap_err();
    assert!(error.is::<wasmparser::BinaryReaderError>());
}
