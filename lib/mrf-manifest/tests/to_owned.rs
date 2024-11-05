use mrf_manifest::Manifest;

const MANIFEST: &str = include_str!("test-manifest.json");

#[test]
fn to_owned_works() {
    let borrowed_manifest: Manifest<'_> = sonic_rs::from_str(MANIFEST).unwrap();
    let owned_manifest: Manifest<'static> = borrowed_manifest.to_owned();
    assert_eq!(borrowed_manifest, owned_manifest);
}
