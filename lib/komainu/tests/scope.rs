use compact_str::CompactString;
use komainu::scope::Scope;
use rstest::rstest;

#[rstest]
#[case("read", "read write")]
#[case("read write", "read write")]
#[case("read write follow", "read write follow push")]
fn can_perform(#[case] request: &str, #[case] client: &str) {
    let request: Scope = request.parse().unwrap();
    let client: Scope = client.parse().unwrap();

    assert!(client.can_perform(&request));
}

#[rstest]
#[case("read write", "read")]
#[case("read follow", "write")]
#[case("write push", "read")]
fn cant_perform(#[case] request: &str, #[case] client: &str) {
    let request: Scope = request.parse().unwrap();
    let client: Scope = client.parse().unwrap();

    assert!(!client.can_perform(&request));
}

#[rstest]
#[case("read", "read write")]
#[case("read", "read")]
#[case("follow", "read follow")]
#[case("write follow", "follow write")]
fn can_access(#[case] endpoint: &str, #[case] client: &str) {
    let endpoint: Scope = endpoint.parse().unwrap();
    let client: Scope = client.parse().unwrap();

    assert!(endpoint.can_be_accessed_by(&client));
}

#[rstest]
#[case("read write", "write")]
#[case("follow", "read write")]
#[case("write follow", "read follow")]
fn cant_access(#[case] endpoint: &str, #[case] client: &str) {
    let endpoint: Scope = endpoint.parse().unwrap();
    let client: Scope = client.parse().unwrap();

    assert!(!endpoint.can_be_accessed_by(&client));
}

#[test]
fn display_impl() {
    let raw_scopes = "read write follow push";
    let scope: Scope = raw_scopes.parse().unwrap();

    assert_eq!(scope.to_string(), raw_scopes);
}

#[test]
fn iter_impls() {
    let raw_scopes = "read write follow";
    let scope: Scope = raw_scopes.parse().unwrap();

    let borrowed_iter: Vec<String> = scope.iter().map(ToString::to_string).collect();
    let owned_iter: Vec<CompactString> = scope.into_iter().collect();

    assert_eq!(borrowed_iter, owned_iter);
    assert_eq!(borrowed_iter, ["read", "write", "follow"]);
}
