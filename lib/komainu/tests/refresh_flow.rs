use self::fixtures::{Fixture, RNG_SEED, TokenValue, generate_secret};
use komainu::{
    ClientExtractor,
    flow::{self, SuccessTokenResponse, TokenType},
    scope::Scope,
};

mod fixtures;

#[futures_test::test]
async fn success_basic() {
    fastrand::seed(RNG_SEED);

    let fixtures = Fixture::generate();
    let client = fixtures
        .client_extractor
        .extract("client_1", None)
        .await
        .unwrap();

    let original_access_token = generate_secret();
    let refresh_token = generate_secret();

    fixtures.token_storage.insert(TokenValue {
        authorization: komainu::Authorization {
            code: generate_secret().into(),
            client: client.clone().into_owned(),
            pkce_payload: None,
            scopes: Scope::new(),
            user_id: "user_id".into(),
        },
        response: SuccessTokenResponse {
            access_token: original_access_token.clone().into(),
            token_type: TokenType::Bearer,
            refresh_token: refresh_token.clone().into(),
            expires_in: 1337,
        },
    });

    let body = sonic_rs::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,

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

    let response = flow::refresh::perform(&req, &fixtures.client_extractor, &fixtures.refresh_flow)
        .await
        .unwrap();

    let success_response = response.unwrap_success();
    assert_ne!(success_response.access_token, original_access_token);

    insta::assert_json_snapshot!(success_response);
}

#[futures_test::test]
async fn invalid_grant_type() {
    fastrand::seed(RNG_SEED);

    let fixtures = Fixture::generate();
    let client = fixtures
        .client_extractor
        .extract("client_1", None)
        .await
        .unwrap();

    let body = sonic_rs::json!({
        "grant_type": "other grant type",
        "refresh_token": generate_secret(),

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

    let error = match flow::refresh::perform(
        &req,
        &fixtures.client_extractor,
        &fixtures.refresh_flow,
    )
    .await
    {
        Ok(..) => unreachable!(),
        Err(err) => err,
    };

    assert!(matches!(error, flow::Error::UnsupportedGrantType));
}
