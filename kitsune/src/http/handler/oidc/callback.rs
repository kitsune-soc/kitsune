use crate::{
    error::{ApiError, Result},
    service::{
        oidc::OidcService,
        user::{Register, UserService},
    },
};
use axum::extract::{Query, State};
use kitsune_db::entity::{prelude::Users, users};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn get(
    State(db_conn): State<DatabaseConnection>,
    State(user_service): State<UserService>,
    State(oidc_service): State<Option<OidcService>>,
    Query(query): Query<CallbackQuery>,
) -> Result<()> {
    let Some(oidc_service) = oidc_service else {
        return Err(ApiError::BadRequest.into());
    };

    let user_info = oidc_service.get_user_info(query.state, query.code).await?;
    let user = if let Some(user) = Users::find()
        .filter(users::Column::Username.eq(user_info.username.as_str()))
        .one(&db_conn)
        .await?
    {
        user
    } else {
        let register = Register::builder()
            .email(user_info.email)
            .username(user_info.username)
            .build()
            .unwrap();

        user_service.register(register).await?
    };

    // TODO: Create internal authorisation code and redirect back to the user

    Ok(())
}
