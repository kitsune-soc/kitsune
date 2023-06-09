use super::url::UrlService;
use crate::{
    error::{Error, Oauth2Error, Result},
    util::generate_secret,
};
use askama::Template;
use askama_axum::IntoResponse;
use async_trait::async_trait;
use axum::response::{Redirect, Response};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use futures_util::FutureExt;
use kitsune_db::{
    model::oauth2,
    schema::{
        oauth2_access_tokens, oauth2_applications, oauth2_authorization_codes,
        oauth2_refresh_tokens,
    },
    PgPool,
};
use oxide_auth::{
    endpoint::{
        OAuthError, OwnerConsent, PreGrant, QueryParameter, Scope, Scopes, Solicitation, WebRequest,
    },
    primitives::{
        grant::{Extensions, Grant},
        issuer::{RefreshedToken, TokenType},
        prelude::IssuedToken,
        registrar::{BoundClient, ClientUrl, ExactUrl, RegisteredUrl, RegistrarError},
    },
};
use oxide_auth_async::{
    endpoint::{Endpoint, OwnerSolicitor},
    primitives::{Authorizer, Issuer, Registrar},
};
use oxide_auth_axum::{OAuthRequest, OAuthResponse, WebError};
use std::{
    borrow::Cow,
    future,
    str::{self, FromStr},
};
use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames, IntoEnumIterator};
use time::{Duration, OffsetDateTime};
use typed_builder::TypedBuilder;
use url::Url;
use uuid::Uuid;

pub static TOKEN_VALID_DURATION: Duration = Duration::hours(1);

/// If the Redirect URI is equal to this string, show the token instead of redirecting the user
const SHOW_TOKEN_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

#[derive(AsRefStr, Clone, Copy, Debug, EnumIter, EnumString, EnumVariantNames)]
#[strum(serialize_all = "lowercase", use_phf)]
pub enum OAuthScope {
    #[strum(serialize = "admin:read")]
    AdminRead,
    #[strum(serialize = "admin:write")]
    AdminWrite,
    Read,
    Write,
}

#[derive(Clone, TypedBuilder)]
pub struct AuthorisationCode {
    application: oauth2::Application,
    scopes: Scope,
    state: Option<String>,
    user_id: Uuid,
}

#[derive(Template)]
#[template(path = "oauth/consent.html")]
struct ConsentPage<'a> {
    app_name: &'a str,
}

#[derive(Clone, TypedBuilder)]
pub struct CreateApp {
    name: String,
    redirect_uris: String,
}

#[derive(Template)]
#[template(path = "oauth/token.html")]
struct ShowTokenPage {
    app_name: String,
    domain: String,
    token: String,
}

#[derive(Clone, TypedBuilder)]
pub struct Oauth2Service {
    db_conn: PgPool,
    url_service: UrlService,
}

impl Oauth2Service {
    pub async fn create_app(&self, create_app: CreateApp) -> Result<oauth2::Application> {
        let mut db_conn = self.db_conn.get().await?;

        diesel::insert_into(oauth2_applications::table)
            .values(oauth2::NewApplication {
                id: Uuid::now_v7(),
                secret: generate_secret().as_str(),
                name: create_app.name.as_str(),
                redirect_uri: create_app.redirect_uris.as_str(),
                scopes: "",
                website: None,
            })
            .get_result(&mut db_conn)
            .await
            .map_err(Error::from)
    }

    pub async fn create_authorisation_code_response(
        &self,
        AuthorisationCode {
            application,
            scopes,
            state,
            user_id,
        }: AuthorisationCode,
    ) -> Result<Response> {
        let mut db_conn = self.db_conn.get().await?;
        let authorization_code: oauth2::AuthorizationCode =
            diesel::insert_into(oauth2_authorization_codes::table)
                .values(oauth2::NewAuthorizationCode {
                    code: generate_secret().as_str(),
                    application_id: application.id,
                    user_id,
                    scopes: scopes.to_string().as_str(),
                    expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                })
                .get_result(&mut db_conn)
                .await?;

        if application.redirect_uri == SHOW_TOKEN_URI {
            Ok(ShowTokenPage {
                app_name: application.name,
                domain: self.url_service.domain().into(),
                token: authorization_code.code,
            }
            .into_response())
        } else {
            let mut url = Url::from_str(&application.redirect_uri)?;
            url.query_pairs_mut()
                .append_pair("code", &authorization_code.code);

            if let Some(state) = state {
                url.query_pairs_mut().append_pair("state", &state);
            }

            Ok(Redirect::to(url.as_str()).into_response())
        }
    }
}

#[derive(Clone)]
pub struct OauthEndpoint {
    authorizer: OauthAuthorizer,
    issuer: OauthIssuer,
    owner_solicitor: OauthOwnerSolicitor,
    registrar: OauthRegistrar,
    scopes: Vec<Scope>,
}

impl From<PgPool> for OauthEndpoint {
    fn from(db_pool: PgPool) -> Self {
        let authorizer = OauthAuthorizer {
            db_pool: db_pool.clone(),
        };
        let issuer = OauthIssuer {
            db_pool: db_pool.clone(),
        };
        let owner_solicitor = OauthOwnerSolicitor {
            db_pool: db_pool.clone(),
        };
        let registrar = OauthRegistrar { db_pool };
        let scopes = OAuthScope::iter()
            .map(|scope| scope.as_ref().parse().unwrap())
            .collect();

        Self {
            authorizer,
            issuer,
            owner_solicitor,
            registrar,
            scopes,
        }
    }
}

impl Endpoint<OAuthRequest> for OauthEndpoint {
    type Error = Oauth2Error;

    fn registrar(&self) -> Option<&(dyn Registrar + Sync)> {
        Some(&self.registrar)
    }

    fn authorizer_mut(&mut self) -> Option<&mut (dyn Authorizer + Send)> {
        Some(&mut self.authorizer)
    }

    fn issuer_mut(&mut self) -> Option<&mut (dyn Issuer + Send)> {
        Some(&mut self.issuer)
    }

    fn owner_solicitor(&mut self) -> Option<&mut (dyn OwnerSolicitor<OAuthRequest> + Send)> {
        Some(&mut self.owner_solicitor)
    }

    fn scopes(&mut self) -> Option<&mut dyn Scopes<OAuthRequest>> {
        Some(&mut self.scopes)
    }

    fn response(
        &mut self,
        _request: &mut OAuthRequest,
        _kind: oxide_auth::endpoint::Template<'_>,
    ) -> Result<<OAuthRequest as WebRequest>::Response, Self::Error> {
        // Idk if thats correct. Just gotta try i guess??
        Ok(OAuthResponse::default())
    }

    fn error(&mut self, err: OAuthError) -> Self::Error {
        err.into()
    }

    fn web_error(&mut self, err: <OAuthRequest as WebRequest>::Error) -> Self::Error {
        err.into()
    }
}

#[derive(Clone)]
struct OauthAuthorizer {
    db_pool: PgPool,
}

#[async_trait]
impl Authorizer for OauthAuthorizer {
    async fn authorize(&mut self, grant: Grant) -> Result<String, ()> {
        let application_id = grant.client_id.parse().map_err(|_| ())?;
        let user_id = grant.owner_id.parse().map_err(|_| ())?;
        let scopes = grant.scope.to_string();
        let expired_at = OffsetDateTime::from_unix_timestamp(grant.until.timestamp())
            .unwrap()
            .replace_nanosecond(grant.until.timestamp_subsec_nanos())
            .unwrap();

        let mut db_conn = self.db_pool.get().await.map_err(|_| ())?;
        diesel::insert_into(oauth2_authorization_codes::table)
            .values(oauth2::NewAuthorizationCode {
                code: generate_secret().as_str(),
                application_id,
                user_id,
                scopes: scopes.as_str(),
                expired_at,
            })
            .returning(oauth2_authorization_codes::code)
            .get_result(&mut db_conn)
            .await
            .map_err(|_| ())
    }

    async fn extract(&mut self, authorization_code: &str) -> Result<Option<Grant>, ()> {
        let mut conn = self.db_pool.get().await.map_err(|_| ())?;
        let oauth_data = oauth2_authorization_codes::table
            .find(authorization_code)
            .inner_join(oauth2_applications::table)
            .first::<(oauth2::AuthorizationCode, oauth2::Application)>(&mut conn)
            .await
            .optional()
            .map_err(|_| ())?;

        let oauth_data = oauth_data.map(|(code, app)| {
            let scope = app.scopes.parse().unwrap();
            let redirect_uri = app.redirect_uri.parse().unwrap();
            let until = chrono::NaiveDateTime::from_timestamp_opt(
                code.expired_at.unix_timestamp(),
                code.expired_at.nanosecond(),
            )
            .unwrap()
            .and_utc();

            Grant {
                owner_id: code.user_id.to_string(),
                client_id: code.application_id.to_string(),
                scope,
                redirect_uri,
                until,
                extensions: Extensions::default(),
            }
        });

        Ok(oauth_data)
    }
}

#[derive(Clone)]
struct OauthIssuer {
    db_pool: PgPool,
}

#[async_trait]
impl Issuer for OauthIssuer {
    async fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
        let application_id = grant.client_id.parse().map_err(|_| ())?;
        let user_id = grant.owner_id.parse().map_err(|_| ())?;
        let scopes = grant.scope.to_string();
        let expired_at = OffsetDateTime::from_unix_timestamp(grant.until.timestamp())
            .unwrap()
            .replace_nanosecond(grant.until.timestamp_subsec_nanos())
            .unwrap();

        let mut db_conn = self.db_pool.get().await.map_err(|_| ())?;
        let (access_token, refresh_token) = db_conn
            .transaction(|tx| {
                async move {
                    let access_token = diesel::insert_into(oauth2_access_tokens::table)
                        .values(oauth2::NewAccessToken {
                            token: generate_secret().as_str(),
                            user_id: Some(user_id),
                            application_id: Some(application_id),
                            scopes: scopes.as_str(),
                            expired_at,
                        })
                        .returning(oauth2::AccessToken::as_returning())
                        .get_result::<oauth2::AccessToken>(tx)
                        .await?;

                    let refresh_token = diesel::insert_into(oauth2_refresh_tokens::table)
                        .values(oauth2::NewRefreshToken {
                            token: generate_secret().as_str(),
                            access_token: access_token.token.as_str(),
                            application_id,
                        })
                        .returning(oauth2::RefreshToken::as_returning())
                        .get_result::<oauth2::RefreshToken>(tx)
                        .await?;

                    Ok::<_, Error>((access_token, refresh_token))
                }
                .scope_boxed()
            })
            .await
            .map_err(|_| ())?;

        Ok(IssuedToken {
            token: access_token.token,
            refresh: Some(refresh_token.token),
            until: grant.until,
            token_type: TokenType::Bearer,
        })
    }

    async fn refresh(&mut self, refresh_token: &str, _grant: Grant) -> Result<RefreshedToken, ()> {
        let mut db_conn = self.db_pool.get().await.map_err(|_| ())?;
        let (refresh_token, access_token) = oauth2_refresh_tokens::table
            .find(refresh_token)
            .inner_join(oauth2_access_tokens::table)
            .select(<(oauth2::RefreshToken, oauth2::AccessToken)>::as_select())
            .get_result::<(oauth2::RefreshToken, oauth2::AccessToken)>(&mut db_conn)
            .await
            .map_err(|_| ())?;

        let (access_token, refresh_token) = db_conn
            .transaction(|tx| {
                async move {
                    let new_access_token = diesel::insert_into(oauth2_access_tokens::table)
                        .values(oauth2::NewAccessToken {
                            user_id: access_token.user_id,
                            token: generate_secret().as_str(),
                            application_id: access_token.application_id,
                            scopes: access_token.scopes.as_str(),
                            expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                        })
                        .get_result::<oauth2::AccessToken>(tx)
                        .await?;

                    let refresh_token = diesel::update(&refresh_token)
                        .set(
                            oauth2_refresh_tokens::access_token.eq(new_access_token.token.as_str()),
                        )
                        .get_result::<oauth2::RefreshToken>(tx)
                        .await?;

                    diesel::delete(&access_token).execute(tx).await?;

                    Ok::<_, Error>((new_access_token, refresh_token))
                }
                .scope_boxed()
            })
            .await
            .map_err(|_| ())?;

        let until = chrono::NaiveDateTime::from_timestamp_opt(
            access_token.expired_at.unix_timestamp(),
            access_token.expired_at.nanosecond(),
        )
        .unwrap()
        .and_utc();

        Ok(RefreshedToken {
            token: access_token.token,
            refresh: Some(refresh_token.token),
            until,
            token_type: TokenType::Bearer,
        })
    }

    async fn recover_token(&mut self, access_token: &str) -> Result<Option<Grant>, ()> {
        let mut db_conn = self.db_pool.get().await.map_err(|_| ())?;
        let oauth_data = oauth2_access_tokens::table
            .find(access_token)
            .inner_join(oauth2_applications::table)
            .select(<(oauth2::AccessToken, oauth2::Application)>::as_select())
            .get_result::<(oauth2::AccessToken, oauth2::Application)>(&mut db_conn)
            .await
            .optional()
            .map_err(|_| ())?;

        let oauth_data = oauth_data.map(|(access_token, app)| {
            let scope = app.scopes.parse().unwrap();
            let redirect_uri = app.redirect_uri.parse().unwrap();
            let until = chrono::NaiveDateTime::from_timestamp_opt(
                access_token.expired_at.unix_timestamp(),
                access_token.expired_at.nanosecond(),
            )
            .unwrap()
            .and_utc();

            Grant {
                owner_id: access_token
                    .user_id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                client_id: app.id.to_string(),
                scope,
                redirect_uri,
                until,
                extensions: Extensions::default(),
            }
        });

        Ok(oauth_data)
    }

    async fn recover_refresh(&mut self, refresh_token: &str) -> Result<Option<Grant>, ()> {
        let mut db_conn = self.db_pool.get().await.map_err(|_| ())?;
        let oauth_data = oauth2_refresh_tokens::table
            .find(refresh_token)
            .inner_join(oauth2_access_tokens::table)
            .inner_join(oauth2_applications::table)
            .select(<(oauth2::AccessToken, oauth2::Application)>::as_select())
            .get_result::<(oauth2::AccessToken, oauth2::Application)>(&mut db_conn)
            .await
            .optional()
            .map_err(|_| ())?;

        let oauth_data = oauth_data.map(|(access_token, app)| {
            let scope = access_token.scopes.parse().unwrap();
            let redirect_uri = app.redirect_uri.parse().unwrap();
            let until = chrono::NaiveDateTime::MAX.and_utc();

            Grant {
                owner_id: access_token
                    .user_id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                client_id: app.id.to_string(),
                scope,
                redirect_uri,
                until,
                extensions: Extensions::default(),
            }
        });

        Ok(oauth_data)
    }
}

#[derive(Clone)]
struct OauthOwnerSolicitor {
    db_pool: PgPool,
}

#[async_trait]
impl OwnerSolicitor<OAuthRequest> for OauthOwnerSolicitor {
    async fn check_consent(
        &mut self,
        req: &mut OAuthRequest,
        solicitation: Solicitation<'_>,
    ) -> OwnerConsent<OAuthResponse> {
        let result = (|| {
            let login_consent = {
                let query = match req.query() {
                    Ok(query) => query,
                    Err(err) => return future::ready(Err(err)).left_future(),
                };

                query.unique_value("login_consent").map(Cow::into_owned)
            };

            async move {
                let consent = match login_consent.as_deref() {
                    Some("accept") => {
                        let body = req.body().ok_or(WebError::Body)?;
                        let user_id = body.unique_value("user_id").ok_or(WebError::Body)?;

                        OwnerConsent::Authorized(user_id.into_owned())
                    }
                    Some("deny") => OwnerConsent::Denied,
                    Some(..) | None => {
                        let client_id: Uuid = solicitation
                            .pre_grant()
                            .client_id
                            .parse()
                            .map_err(|_| WebError::Endpoint(OAuthError::BadRequest))?;

                        let mut db_conn = self
                            .db_pool
                            .get()
                            .await
                            .map_err(|_| WebError::InternalError(None))?;

                        let app_name = oauth2_applications::table
                            .find(client_id)
                            .select(oauth2_applications::name)
                            .get_result::<String>(&mut db_conn)
                            .await
                            .optional()
                            .map_err(|_| WebError::InternalError(None))?
                            .ok_or(WebError::Endpoint(OAuthError::DenySilently))?;

                        let body = ConsentPage {
                            app_name: &app_name,
                        }
                        .render()
                        .map_err(|err| WebError::InternalError(Some(err.to_string())))?;

                        OwnerConsent::InProgress(
                            OAuthResponse::default()
                                .content_type("text/html")
                                .unwrap()
                                .body(&body),
                        )
                    }
                };

                Ok(consent)
            }
            .right_future()
        })()
        .await;

        match result {
            Ok(consent) => consent,
            Err(err) => OwnerConsent::Error(err),
        }
    }
}

#[derive(Clone)]
struct OauthRegistrar {
    db_pool: PgPool,
}

#[async_trait]
impl Registrar for OauthRegistrar {
    async fn bound_redirect<'a>(
        &self,
        bound: ClientUrl<'a>,
    ) -> Result<BoundClient<'a>, RegistrarError> {
        if let Some(redirect_uri) = bound.redirect_uri {
            Ok(BoundClient {
                client_id: bound.client_id,
                redirect_uri: Cow::Owned(RegisteredUrl::Exact(redirect_uri.into_owned())),
            })
        } else {
            Err(RegistrarError::Unspecified)
        }
    }

    async fn negotiate<'a>(
        &self,
        client: BoundClient<'a>,
        scope: Option<Scope>,
    ) -> Result<PreGrant, RegistrarError> {
        let client_id: Uuid = client
            .client_id
            .parse()
            .map_err(|_| RegistrarError::PrimitiveError)?;

        let mut db_conn = self
            .db_pool
            .get()
            .await
            .map_err(|_| RegistrarError::PrimitiveError)?;

        let client = oauth2_applications::table
            .find(client_id)
            .filter(oauth2_applications::redirect_uri.eq(client.redirect_uri.as_str()))
            .get_result::<oauth2::Application>(&mut db_conn)
            .await
            .optional()
            .map_err(|_| RegistrarError::PrimitiveError)?
            .ok_or(RegistrarError::Unspecified)?;

        let client_id = client.id.to_string();
        let redirect_uri = ExactUrl::new(client.redirect_uri)
            .map_err(|_| RegistrarError::PrimitiveError)?
            .into();

        let scope = if let Some(scope) = scope {
            let valid_scopes: Vec<&str> = scope
                .iter()
                .filter(|scope| OAuthScope::from_str(scope).is_ok())
                .collect();

            if valid_scopes.is_empty() {
                OAuthScope::Read.as_ref().parse().unwrap()
            } else {
                valid_scopes.join(" ").parse().unwrap()
            }
        } else {
            OAuthScope::Read.as_ref().parse().unwrap()
        };

        Ok(PreGrant {
            client_id,
            redirect_uri,
            scope,
        })
    }

    async fn check(
        &self,
        client_id: &str,
        passphrase: Option<&[u8]>,
    ) -> Result<(), RegistrarError> {
        let client_id: Uuid = client_id
            .parse()
            .map_err(|_| RegistrarError::PrimitiveError)?;
        let mut client_query = oauth2_applications::table.find(client_id).into_boxed();

        if let Some(passphrase) = passphrase {
            let passphrase =
                str::from_utf8(passphrase).map_err(|_| RegistrarError::PrimitiveError)?;
            client_query = client_query.filter(oauth2_applications::secret.eq(passphrase));
        }

        let mut db_conn = self
            .db_pool
            .get()
            .await
            .map_err(|_| RegistrarError::PrimitiveError)?;

        client_query
            .select(oauth2_applications::id)
            .execute(&mut db_conn)
            .await
            .optional()
            .map_err(|_| RegistrarError::PrimitiveError)?
            .map(|_| ())
            .ok_or(RegistrarError::Unspecified)
    }
}
