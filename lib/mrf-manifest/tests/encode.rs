#![cfg(feature = "encode")]

const MANIFEST: &str = include_str!("test-manifest.json");

#[test]
fn encode_works() {
    let manifest = serde_json::from_str(MANIFEST).unwrap();
    let encoded_manifest = mrf_manifest::encode(&manifest).unwrap();
    insta::assert_json_snapshot!(encoded_manifest);
}
