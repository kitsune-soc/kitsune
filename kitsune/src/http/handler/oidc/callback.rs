use crate::{
    error::{ApiError, Result},
    service::{
        oauth2::{AuthorisationCode, OAuth2Service},
        oidc::OidcService,
        user::{Register, UserService},
    },
};
use axum::{
    extract::{Query, State},
    response::Response,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    schema::{oauth2_applications, users},
    PgPool,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn get(
    State(db_pool): State<PgPool>,
    State(user_service): State<UserService>,
    State(oauth_service): State<OAuth2Service>,
    State(oidc_service): State<Option<OidcService>>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response> {
    let Some(oidc_service) = oidc_service else {
        return Err(ApiError::BadRequest.into());
    };

    let user_info = oidc_service.get_user_info(query.state, query.code).await?;
    let user = db_pool
        .with_connection(|mut db_conn| {
            let user_info = &user_info;

            async move {
                users::table
                    .filter(users::oidc_id.eq(&user_info.subject))
                    .get_result(&mut db_conn)
                    .await
                    .optional()
            }
        })
        .await?;

    let user = if let Some(user) = user {
        user
    } else {
        let register = Register::builder()
            .email(user_info.email)
            .username(user_info.username)
            .oidc_id(user_info.subject)
            .build();

        user_service.register(register).await?
    };

    let application = db_pool
        .with_connection(|mut db_conn| {
            oauth2_applications::table
                .find(user_info.oauth2.application_id)
                .get_result(&mut db_conn)
        })
        .await?;

    let authorisation_code = AuthorisationCode::builder()
        .application(application)
        .state(user_info.oauth2.state)
        .user_id(user.id)
        .scopes(user_info.oauth2.scope.parse()?)
        .build();

    oauth_service
        .create_authorisation_code_response(authorisation_code)
        .await
}
