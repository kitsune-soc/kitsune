use crate::oauth2::{ClientExtractor, CodeGrantIssuer, OAuthScope, SHOW_TOKEN_URI};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    debug_handler,
    extract::{OriginalUri, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_extra::{
    either::{Either, Either3},
    extract::{
        cookie::{Cookie, Expiration, SameSite},
        SignedCookieJar,
    },
};
use cursiv::{CsrfHandle, MessageRef};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use flashy::{FlashHandle, IncomingFlashes};
use kitsune_db::{
    model::user::User, schema::oauth2_applications, schema::users, with_connection, PgPool,
};
use kitsune_error::{kitsune_error, Error, ErrorType, Result};
use kitsune_url::UrlService;
use komainu::code_grant::{self, Authorizer};
use serde::Deserialize;
use speedy_uuid::Uuid;
use std::{
    borrow::{Borrow, Cow},
    str::FromStr,
};

const UNCONFIRMED_EMAIL_ADDRESS: &str = "Email address is unconfirmed. Check your inbox!";
const WRONG_EMAIL_OR_PASSWORD: &str = "Entered wrong email or password";

#[cfg(feature = "oidc")]
use {axum::extract::Query, kitsune_db::model::oauth2, kitsune_oidc::OidcService};

#[cfg(feature = "oidc")]
#[derive(Deserialize)]
pub struct AuthorizeQuery {
    client_id: Uuid,
    redirect_uri: String,
    scope: String,
    state: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[instrument(skip_all)]
fn build_login_page<I>(
    authenticated_user: &User,
    app_name: &str,
    scopes: &[OAuthScope],
    user_id: &str,

    authorizer: &Authorizer<'_, I>,
    csrf_handle: &CsrfHandle,
) -> Html<String>
where
    I: code_grant::Issuer,
{
    let csrf_token = csrf_handle.sign(user_id); // TODO: BAD DO NOT USE USER-ID

    let client_id = authorizer.query().get("client_id");
    let redirect_uri = authorizer.query().get("redirect_uri");
    let response_type = authorizer.query().get("response_type");
    let scope = authorizer.query().get("scope");
    let state = authorizer.query().get("state");

    let body = crate::template::render(
        "oauth/consent.html",
        minijinja::context! {
            authenticated_username => &authenticated_user.username,
            app_name => &app_name,
            csrf_token => csrf_token.as_str(),
            query => minijinja::context! {
                client_id => client_id,
                redirect_uri => redirect_uri,
                response_type => response_type,
                scope => scope,
                state => state,
            },
            scopes => &scopes,
        },
    )
    .unwrap();

    Html(body)
}

#[instrument(skip_all)]
async fn process_consent<I>(
    authenticated_user: &User,
    app_name: &str,
    scopes: &[OAuthScope],

    authorizer: Authorizer<'_, I>,
    consent: Cow<'_, str>,
    url_service: &UrlService,
) -> Result<axum::response::Response>
where
    I: code_grant::Issuer<UserId = Uuid>,
{
    match consent.borrow() {
        "accept" => {
            let redirect_uri = authorizer.client().redirect_uri.clone();
            let scopes = scopes.iter().map(AsRef::as_ref).collect();
            let acceptor = match authorizer.accept(authenticated_user.id, &scopes).await {
                Ok(acceptor) => acceptor,
                Err(response) => return Ok(response.map(|()| axum::body::Body::empty())),
            };

            if redirect_uri == SHOW_TOKEN_URI {
                let page = crate::template::render(
                    "oauth/token.html",
                    minijinja::context! {
                        app_name => app_name,
                        domain => url_service.domain(),
                        token => acceptor.code(),
                    },
                )
                .unwrap();

                Ok(Html(page).into_response())
            } else {
                Ok(acceptor.into_response().map(|()| axum::body::Body::empty()))
            }
        }
        "deny" => Ok(authorizer.deny().map(|()| axum::body::Body::empty())),
        _ => return Err(kitsune_error!(type = ErrorType::BadRequest, "invalid consent param")),
    }
}

#[cfg_attr(feature = "oidc", debug_handler(state = crate::state::Zustand))]
pub async fn get(
    #[cfg(feature = "oidc")] (State(oidc_service), Query(query)): (
        State<Option<OidcService>>,
        Query<AuthorizeQuery>,
    ),

    State(db_pool): State<PgPool>,
    State(url_service): State<UrlService>,
    cookies: SignedCookieJar,
    csrf_handle: CsrfHandle,
    flash_messages: IncomingFlashes,

    request: axum::extract::Request,
) -> Result<Either3<Html<String>, axum::response::Response, Redirect>> {
    #[cfg(feature = "oidc")]
    if let Some(oidc_service) = oidc_service {
        let application = with_connection!(db_pool, |db_conn| {
            oauth2_applications::table
                .find(query.client_id)
                .filter(oauth2_applications::redirect_uri.eq(query.redirect_uri))
                .get_result::<oauth2::Application>(db_conn)
                .await
        })?;

        let auth_url = oidc_service
            .authorisation_url(application.id, query.scope, query.state)
            .await?;

        return Ok(Either3::E3(Redirect::to(auth_url.as_str())));
    }

    let authenticated_user = if let Some(user_id) = cookies.get("user_id") {
        let id = user_id.value().parse::<Uuid>()?;
        with_connection!(db_pool, |db_conn| {
            users::table.find(id).get_result::<User>(db_conn).await
        })?
    } else {
        let messages: Vec<(flashy::Level, &str)> = flash_messages.into_iter().collect();
        let page = crate::template::render(
            "oauth/login.html",
            minijinja::context! {
                flash_messages => messages,
            },
        )
        .unwrap();

        return Ok(Either3::E1(Html(page)));
    };

    // ToDo: move into state
    let extractor = komainu::code_grant::AuthorizerExtractor::new(
        CodeGrantIssuer::builder().db_pool(db_pool.clone()).build(),
        ClientExtractor::builder().db_pool(db_pool.clone()).build(),
    );

    let request = komainu::Request::read_from(request).await?;
    let authorizer = extractor.extract_raw(&request).await?;

    let client_id: Uuid = authorizer.client().client_id.parse()?;

    let app_name = with_connection!(db_pool, |db_conn| {
        oauth2_applications::table
            .find(client_id)
            .select(oauth2_applications::name)
            .get_result::<String>(db_conn)
            .await
    })?;

    let mut scopes = authorizer
        .scope()
        .iter()
        .filter_map(|scope| OAuthScope::from_str(scope).ok())
        .collect::<Vec<OAuthScope>>();

    if scopes.is_empty() {
        // default to read scope if no scopes are defined
        scopes.push(OAuthScope::Read);
    }

    let user_id = authenticated_user.id.to_string();

    if let Some(consent) = authorizer.query().get("login_consent") {
        let csrf_token = authorizer
            .query()
            .get("csrf_token")
            .ok_or_else(|| kitsune_error!("missing csrf token"))?;

        if !csrf_handle.verify(MessageRef::from_str(csrf_token.borrow())) {
            return Err(kitsune_error!(type = ErrorType::Forbidden, "invalid csrf token"));
        }

        let consent = consent.clone();
        let response = process_consent(
            &authenticated_user,
            &app_name,
            &scopes,
            authorizer,
            consent,
            &url_service,
        )
        .await?;

        Ok(Either3::E2(response))
    } else {
        let page = build_login_page(
            &authenticated_user,
            &app_name,
            &scopes,
            &user_id,
            &authorizer,
            &csrf_handle,
        );

        Ok(Either3::E1(page))
    }
}

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(db_pool): State<PgPool>,
    OriginalUri(original_url): OriginalUri,
    cookies: SignedCookieJar,
    flash_handle: FlashHandle,
    Form(form): Form<LoginForm>,
) -> Result<Either<(SignedCookieJar, Redirect), Redirect>> {
    let redirect_to = if let Some(path_and_query) = original_url.path_and_query() {
        path_and_query.as_str()
    } else {
        original_url.path()
    };

    let user = with_connection!(db_pool, |db_conn| {
        users::table
            .filter(users::username.eq(form.username))
            .first::<User>(db_conn)
            .await
            .optional()
    })?;

    let Some(user) = user else {
        flash_handle.push(flashy::Level::Error, WRONG_EMAIL_OR_PASSWORD);
        return Ok(Either::E2(Redirect::to(redirect_to)));
    };

    if user.confirmed_at.is_none() {
        flash_handle.push(flashy::Level::Error, UNCONFIRMED_EMAIL_ADDRESS);
        return Ok(Either::E2(Redirect::to(redirect_to)));
    }

    let is_valid = blowocking::crypto(move || {
        let password_hash = PasswordHash::new(user.password.as_ref().unwrap())?;
        let argon2 = Argon2::default();

        Ok::<_, Error>(
            argon2
                .verify_password(form.password.as_bytes(), &password_hash)
                .is_ok(),
        )
    })
    .await??;

    if !is_valid {
        flash_handle.push(flashy::Level::Error, WRONG_EMAIL_OR_PASSWORD);
        return Ok(Either::E2(Redirect::to(redirect_to)));
    }

    // TODO: Bad because no expiration. Either encode an expiration into the cookie and make this basically a shitty JWT
    // or store session IDs instead that are stored in a TTL data structure (these would need to be either storable in-memory or in Redis; similar to OIDC data)
    let user_id_cookie = Cookie::build(("user_id", user.id.to_string()))
        .same_site(SameSite::Strict)
        .expires(Expiration::Session)
        .secure(true);

    Ok(Either::E1((
        cookies.add(user_id_cookie),
        Redirect::to(redirect_to),
    )))
}
