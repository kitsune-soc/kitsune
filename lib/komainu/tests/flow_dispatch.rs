use self::{
    fixtures::{Fixture, RNG_SEED, TokenValue, generate_secret},
    util::SerdeResponse,
};
use komainu::{
    ClientExtractor,
    flow::{self, SuccessTokenResponse, TokenType},
    scope::Scope,
};

mod fixtures;
mod util;

#[futures_test::test]
async fn auth_code_success() {
    fastrand::seed(RNG_SEED);

    let fixture = Fixture::generate();
    let impls = flow::Impls {
        auth_issuer: fixture.auth_flow.clone(),
        client_extractor: fixture.client_extractor.clone(),
        refresh_issuer: fixture.refresh_flow.clone(),
    };

    let code = generate_secret();
    let client = fixture
        .client_extractor
        .extract("client_1", None)
        .await
        .unwrap();

    fixture.auth_storage.insert(komainu::Authorization {
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

    let response = flow::dispatch(&req, &impls).await.unwrap();
    insta::assert_json_snapshot!(SerdeResponse::from(response));
}

#[futures_test::test]
async fn refrersh_token_success() {
    fastrand::seed(RNG_SEED);

    let fixtures = Fixture::generate();
    let impls = flow::Impls {
        auth_issuer: fixtures.auth_flow.clone(),
        client_extractor: fixtures.client_extractor.clone(),
        refresh_issuer: fixtures.refresh_flow.clone(),
    };

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

    let response = flow::dispatch(&req, &impls).await.unwrap();
    insta::assert_json_snapshot!(SerdeResponse::from(response));
}

#[futures_test::test]
async fn unknown_grant_type() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let impls = flow::Impls {
        auth_issuer: fixture.auth_flow.clone(),
        client_extractor: fixture.client_extractor.clone(),
        refresh_issuer: fixture.refresh_flow.clone(),
    };

    let body = sonic_rs::json!({
        "grant_type": "invalid shit"
    });
    let body = sonic_rs::to_string(&body).unwrap();

    let request = http::Request::builder()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .unwrap();
    let request = komainu::Request::read_from(request).await.unwrap();

    let response = flow::dispatch(&request, &impls).await.unwrap();
    insta::assert_json_snapshot!(SerdeResponse::from(response));
}
