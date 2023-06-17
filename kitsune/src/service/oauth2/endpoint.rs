use super::{
    authorizer::OAuthAuthorizer, issuer::OAuthIssuer, registrar::OAuthRegistrar, OAuthScope,
};
use crate::error::OAuth2Error;
use kitsune_db::PgPool;
use oxide_auth::endpoint::{OAuthError, OwnerConsent, Scope, Scopes, Solicitation, WebRequest};
use oxide_auth_async::{
    endpoint::{Endpoint, OwnerSolicitor},
    primitives::{Authorizer, Issuer, Registrar},
};
use oxide_auth_axum::{OAuthRequest, OAuthResponse};
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct OAuthEndpoint<S = Vacant> {
    authorizer: OAuthAuthorizer,
    issuer: OAuthIssuer,
    owner_solicitor: S,
    registrar: OAuthRegistrar,
    scopes: Vec<Scope>,
}

impl<S> OAuthEndpoint<S> {
    pub fn with_solicitor<NewSolicitor>(
        self,
        owner_solicitor: NewSolicitor,
    ) -> OAuthEndpoint<NewSolicitor>
    where
        NewSolicitor: OwnerSolicitor<OAuthRequest> + Send,
    {
        OAuthEndpoint {
            authorizer: self.authorizer,
            issuer: self.issuer,
            owner_solicitor,
            registrar: self.registrar,
            scopes: self.scopes,
        }
    }
}

impl From<PgPool> for OAuthEndpoint {
    fn from(db_pool: PgPool) -> Self {
        let authorizer = OAuthAuthorizer {
            db_pool: db_pool.clone(),
        };
        let issuer = OAuthIssuer {
            db_pool: db_pool.clone(),
        };
        let registrar = OAuthRegistrar { db_pool };
        let scopes = OAuthScope::iter()
            .map(|scope| scope.as_ref().parse().unwrap())
            .collect();

        Self {
            authorizer,
            issuer,
            owner_solicitor: Vacant,
            registrar,
            scopes,
        }
    }
}

impl<S> Endpoint<OAuthRequest> for OAuthEndpoint<S>
where
    S: OwnerSolicitor<OAuthRequest> + Send,
{
    type Error = OAuth2Error;

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
        Ok(OAuthResponse::default())
    }

    fn error(&mut self, err: OAuthError) -> Self::Error {
        err.into()
    }

    fn web_error(&mut self, err: <OAuthRequest as WebRequest>::Error) -> Self::Error {
        err.into()
    }
}

#[derive(Clone, Copy)]
pub struct Vacant;

impl<T> oxide_auth::endpoint::OwnerSolicitor<T> for Vacant
where
    T: WebRequest,
{
    fn check_consent(
        &mut self,
        _req: &mut T,
        _solicitation: Solicitation<'_>,
    ) -> OwnerConsent<T::Response> {
        OwnerConsent::Denied
    }
}
