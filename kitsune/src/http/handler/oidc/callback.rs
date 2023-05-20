use crate::{
    error::{ApiError, Result},
    service::{
        oauth2::{AuthorisationCode, Oauth2Service},
        oidc::OidcService,
        user::{Register, UserService},
    },
};
use axum::{
    extract::{Query, State},
    response::Response,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn get(
    State(db_conn): State<DatabaseConnection>,
    State(user_service): State<UserService>,
    State(oauth_service): State<Oauth2Service>,
    State(oidc_service): State<Option<OidcService>>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response> {
    let Some(oidc_service) = oidc_service else {
        return Err(ApiError::BadRequest.into());
    };

    let user_info = oidc_service.get_user_info(query.state, query.code).await?;
    let user = if let Some(user) = Users::find()
        .filter(users::Column::OidcId.eq(&user_info.subject))
        .one(&db_conn)
        .await?
    {
        user
    } else {
        let register = Register::builder()
            .email(user_info.email)
            .username(user_info.username)
            .oidc_id(user_info.subject)
            .build();

        user_service.register(register).await?
    };

    let application = Oauth2Applications::find_by_id(user_info.oauth2.application_id)
        .one(&db_conn)
        .await?
        .ok_or_else(|| {
            error!("OAuth2 application stored inside the login state not available anymore");
            ApiError::InternalServerError
        })?;

    let authorisation_code = AuthorisationCode::builder()
        .application(application)
        .state(user_info.oauth2.state)
        .user_id(user.id)
        .build();

    oauth_service
        .create_authorisation_code_response(authorisation_code)
        .await
}
