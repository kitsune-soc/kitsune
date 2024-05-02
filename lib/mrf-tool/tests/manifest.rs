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
fn read() {}

#[test]
fn remove() {
    let empty = empty_module();
    let module_with_manifest = module_with_manifest();

    let mut fs = DummyFs::default();
    fs.insert("module.wasm".into(), module_with_manifest.clone());

    mrf_tool::remove_manifest(
        &mut fs,
        Path::new("module.wasm"),
        Path::new("module.removed.wasm"),
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
