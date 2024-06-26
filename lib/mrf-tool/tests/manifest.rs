use mrf_manifest::{Manifest, SECTION_NAME};
use mrf_tool::DummyFs;
use std::{io, path::Path, str};

const MANIFEST: &str = include_str!("test-manifest.json");

fn empty_module() -> Vec<u8> {
    wat::parse_str("( module )").unwrap()
}

fn module_with_manifest() -> Vec<u8> {
    let escaped_manifest = manifest_to_escaped_string();
    wat::parse_str(format!(
        r#"( module ( @custom "{SECTION_NAME}" {escaped_manifest} ) )"#
    ))
    .unwrap()
}

fn manifest_to_escaped_string() -> String {
    let manifest: Manifest<'_> = serde_json::from_str(MANIFEST).unwrap();
    let encoded_manifest = mrf_manifest::serialise(&manifest).unwrap();
    serde_json::to_string(str::from_utf8(&encoded_manifest).unwrap()).unwrap()
}

#[test]
fn add() {
    let empty = empty_module();
    let module_with_manifest = module_with_manifest();

    let mut fs = DummyFs::default();
    fs.insert("module.wasm".into(), empty.clone());
    fs.insert("manifest.json".into(), MANIFEST.into());

    mrf_tool::write_manifest(&mut fs, MANIFEST.as_ref(), "module.wasm".as_ref()).unwrap();
    assert_eq!(
        *fs.get(Path::new("module.wasm")).unwrap(),
        module_with_manifest
    );

    fs.remove(Path::new("module.wasm")).unwrap();
    fs.insert("module.wasm".into(), empty);

    mrf_tool::handle(
        &mut fs,
        &mut io::sink(),
        [
            "mrf-tool",
            "manifest",
            "add",
            "manifest.json",
            "module.wasm",
            "--output",
            "module.manifest.wasm",
        ],
    )
    .unwrap();
    assert_eq!(
        *fs.get(Path::new("module.manifest.wasm")).unwrap(),
        module_with_manifest
    );
}

#[test]
fn read() {
    let manifest: Manifest<'_> = serde_json::from_str(MANIFEST).unwrap();
    let module_with_manifest = module_with_manifest();

    let pretty_manifest = colored_json::to_colored_json_auto(&manifest)
        .unwrap()
        .into_bytes();

    let mut sink = Vec::new();
    mrf_tool::read_manifest(&mut sink, &module_with_manifest).unwrap();
    assert_eq!(sink.pop(), Some(b'\n'));
    assert_eq!(sink, pretty_manifest);

    let mut fs = DummyFs::default();
    fs.insert("module.wasm".into(), module_with_manifest);

    let mut sink = Vec::new();
    mrf_tool::handle(
        &mut fs,
        &mut sink,
        ["mrf-tool", "manifest", "read", "module.wasm"],
    )
    .unwrap();

    assert_eq!(sink.pop(), Some(b'\n'));
    assert_eq!(sink, pretty_manifest);
}

#[test]
fn read_errors() {
    let empty_module = empty_module();

    let error = mrf_tool::read_manifest(&mut io::sink(), &empty_module).unwrap_err();
    assert_eq!(error.to_string(), "missing manifest in module");

    let mut fs = DummyFs::default();
    fs.insert("empty.wasm".into(), empty_module);

    let error = mrf_tool::handle(
        &mut fs,
        &mut io::sink(),
        ["mrf-tool", "manifest", "read", "empty.wasm"],
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "missing manifest in module");
}

#[test]
fn remove() {
    let empty = empty_module();
    let module_with_manifest = module_with_manifest();

    let mut fs = DummyFs::default();
    fs.insert("module.wasm".into(), module_with_manifest.clone());

    mrf_tool::remove_manifest(
        &mut fs,
        "module.wasm".as_ref(),
        "module.removed.wasm".as_ref(),
    )
    .unwrap();
    assert_eq!(*fs.get(Path::new("module.removed.wasm")).unwrap(), empty);

    fs.remove(Path::new("module.removed.wasm"));

    mrf_tool::handle(
        &mut fs,
        &mut io::sink(),
        [
            "mrf-tool",
            "manifest",
            "remove",
            "module.wasm",
            "--output",
            "module.removed.wasm",
        ],
    )
    .unwrap();
    assert_eq!(*fs.get(Path::new("module.removed.wasm")).unwrap(), empty);
}

#[test]
fn remove_errors() {
    let mut fs = DummyFs::default();
    fs.insert("empty.wasm".into(), empty_module());

    let error = mrf_tool::remove_manifest(
        &mut fs,
        "empty.wasm".as_ref(),
        "empty.removed.wasm".as_ref(),
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "missing manifest in module");

    let error = mrf_tool::handle(
        &mut fs,
        &mut io::sink(),
        [
            "mrf-tool",
            "manifest",
            "remove",
            "empty.wasm",
            "--output",
            "empty.removed.wasm",
        ],
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "missing manifest in module");
}
