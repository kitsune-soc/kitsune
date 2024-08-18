#![cfg(feature = "encode")]

use mrf_manifest::SECTION_NAME;

const MANIFEST: &str = include_str!("test-manifest.json");

#[test]
fn encode_works() {
    let manifest = sonic_rs::from_str(MANIFEST).unwrap();
    let canonical_manifest =
        String::from_utf8(mrf_manifest::serialise(&manifest).unwrap()).unwrap();
    let encoded_manifest = mrf_manifest::encode(&manifest).unwrap();
    let escaped_manifest = sonic_rs::to_string(&canonical_manifest).unwrap();

    let from_scratch = wat::parse_str(format!(
        r#"(module (@custom "{SECTION_NAME}" {escaped_manifest} ) )"#,
    ))
    .unwrap();

    let mut pasted_together = wat::parse_str("(module)").unwrap();
    pasted_together.extend(encoded_manifest);

    assert_eq!(pasted_together, from_scratch);
}
