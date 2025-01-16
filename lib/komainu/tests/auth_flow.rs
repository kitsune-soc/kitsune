use self::fixtures::{generate_secret, Fixture, RNG_SEED};
use komainu::{flow, scope::Scope, ClientExtractor};

mod fixtures;

#[futures_test::test]
async fn success_basic() {
    fastrand::seed(RNG_SEED);

    let fixtures = Fixture::generate();
    let code = generate_secret();
    let client = fixtures
        .client_extractor
        .extract("client_1", None)
        .await
        .unwrap();

    fixtures.auth_storage.insert(komainu::Authorization {
        code: code.clone().into(),
        client: client.clone(),
        pkce_payload: None,
        scopes: Scope::new(),
        user_id: "user_id".into(),
    });

    let body = sonic_rs::json!({
        "grant_type": "authorization_code",
        "code": code,

        "client_id": client.client_id,
        "client_secret": client.client_secret,
        "redirect_uri": client.redirect_uri,
    });
    let body = sonic_rs::to_string(&body).unwrap();

    let req = http::Request::builder()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .unwrap();
    let req = komainu::Request::read_from(req).await.unwrap();

    let response =
        flow::authorization::perform(&req, &fixtures.client_extractor, &fixtures.auth_flow)
            .await
            .unwrap();

    insta::assert_json_snapshot!(response);
}
