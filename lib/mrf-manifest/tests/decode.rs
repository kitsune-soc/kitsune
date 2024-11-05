#![cfg(feature = "decode")]

use mrf_manifest::{Manifest, SECTION_NAME};

const MANIFEST: &str = include_str!("test-manifest.json");

#[test]
fn decode_works() {
    let manifest: Manifest<'_> = sonic_rs::from_str(MANIFEST).unwrap();
    // Calling `sonic_rs::to_string` on a string will encode it into its JSON representation
    // which is a fully escaped representation of the string
    let escaped_manifest = sonic_rs::to_string(&MANIFEST).unwrap();

    let wat_src = format!(r#"( module ( @custom "{SECTION_NAME}" {escaped_manifest} ) )"#);
    let wasm_blob = wat::parse_str(wat_src).unwrap();

    let (parsed_manifest, section_range) = mrf_manifest::decode(&wasm_blob).unwrap().unwrap();
    assert_eq!(manifest, parsed_manifest);
    assert_eq!(section_range, 8..245);
}
