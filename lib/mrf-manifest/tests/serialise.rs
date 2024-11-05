#![cfg(feature = "serialise")]

const MANIFEST: &str = include_str!("test-manifest.json");

#[test]
fn serialise_works() {
    let manifest = sonic_rs::from_str(MANIFEST).unwrap();
    let encoded_manifest = mrf_manifest::serialise(&manifest).unwrap();
    let encoded_manifest_str = String::from_utf8(encoded_manifest).unwrap();
    insta::assert_snapshot!(encoded_manifest_str);
}
