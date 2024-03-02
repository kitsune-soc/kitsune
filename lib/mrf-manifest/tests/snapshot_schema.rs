use mrf_manifest::Manifest;

#[test]
fn json_schema() {
    let schema = schemars::schema_for!(Manifest<'_>);
    insta::assert_json_snapshot!(schema);
}
