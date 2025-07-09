use komainu::scope::Scope;
use std::borrow::Cow;

#[test]
fn only_compares_id_and_secret() {
    let client_1 = komainu::Client {
        client_id: Cow::Borrowed("id"),
        client_secret: Cow::Borrowed("secret"),
        scopes: Scope::from_iter(["read", "write"]),
        redirect_uri: Cow::Borrowed("redirect uri"),
    };

    let mut client_2 = komainu::Client {
        client_id: Cow::Borrowed("id"),
        client_secret: Cow::Borrowed("secret"),
        scopes: Scope::from_iter(["read", "write"]),
        redirect_uri: Cow::Borrowed("redirect uri"),
    };

    assert!(client_1 == client_2);

    client_2.scopes = Scope::from_iter(["follow"]);
    client_2.redirect_uri = Cow::Borrowed("other redirect uri");
    assert!(client_1 == client_2);

    client_2.client_id = Cow::Borrowed("other id");
    assert!(client_1 != client_2);

    client_2.client_id = Cow::Borrowed("id");
    client_2.client_secret = Cow::Borrowed("other secret");
    assert!(client_1 != client_2);
}
