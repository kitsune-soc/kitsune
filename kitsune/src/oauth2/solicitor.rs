use super::OAuthScope;
use async_trait::async_trait;
use cursiv::CsrfHandle;
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::user::User, schema::oauth2_applications, with_connection, PgPool};
use kitsune_error::Result;
use oxide_auth::endpoint::{OAuthError, OwnerConsent, QueryParameter, Solicitation, WebRequest};
use oxide_auth_async::endpoint::OwnerSolicitor;
use oxide_auth_axum::{OAuthRequest, OAuthResponse, WebError};
use speedy_uuid::Uuid;
use std::{borrow::Cow, str::FromStr};
use trials::attempt;
use typed_builder::TypedBuilder;

struct PageQueryParams {
    client_id: String,
    csrf_token: Option<String>,
    redirect_uri: String,
    response_type: String,
    scope: String,
    state: Option<String>,
}

impl PageQueryParams {
    fn extract(query: &(dyn QueryParameter + 'static)) -> Option<Self> {
        Some(Self {
            client_id: query.unique_value("client_id")?.into_owned(),
            csrf_token: query.unique_value("csrf_token").map(Cow::into_owned),
            redirect_uri: query.unique_value("redirect_uri")?.into_owned(),
            response_type: query.unique_value("response_type")?.into_owned(),
            scope: query.unique_value("scope")?.into_owned(),
            state: query.unique_value("state").map(Cow::into_owned),
        })
    }
}

#[derive(Clone, TypedBuilder)]
pub struct OAuthOwnerSolicitor {
    authenticated_user: User,
    csrf_handle: CsrfHandle,
    db_pool: PgPool,
}

impl OAuthOwnerSolicitor {
    #[instrument(skip_all)]
    async fn check_consent(
        &self,
        login_consent: Option<&str>,
        query: PageQueryParams,
        solicitation: &Solicitation<'_>,
    ) -> Result<OwnerConsent<OAuthResponse>, WebError> {
        let consent = match login_consent {
            Some("accept") => {
                let Some(csrf_token) = query.csrf_token else {
                    return Err(WebError::Query);
                };

                if !self.csrf_handle.verify(csrf_token.as_str().into()) {
                    return Err(WebError::Authorization);
                }

                OwnerConsent::Authorized(self.authenticated_user.id.to_string())
            }
            Some("deny") => OwnerConsent::Denied,
            Some(..) | None => {}
        };

        Ok(consent)
    }
}

#[async_trait]
impl OwnerSolicitor<OAuthRequest> for OAuthOwnerSolicitor {
    #[instrument(skip_all)]
    async fn check_consent(
        &mut self,
        req: &mut OAuthRequest,
        solicitation: Solicitation<'_>,
    ) -> OwnerConsent<OAuthResponse> {
        let (login_consent, query) = {
            let query = match req.query() {
                Ok(query) => query,
                Err(err) => return OwnerConsent::Error(err),
            };

            let login_consent = query.unique_value("login_consent").map(Cow::into_owned);
            let Some(query_params) = PageQueryParams::extract(query.as_ref()) else {
                return OwnerConsent::Error(WebError::Endpoint(OAuthError::BadRequest));
            };

            (login_consent, query_params)
        };

        let result =
            Self::check_consent(self, login_consent.as_deref(), query, &solicitation).await;

        match result {
            Ok(consent) => consent,
            Err(err) => OwnerConsent::Error(err),
        }
    }
}
