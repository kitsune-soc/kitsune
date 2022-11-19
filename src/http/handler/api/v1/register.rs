use crate::{db::entity::user, error::Result, state::State};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form,
};
use chrono::Utc;
use rsa::{
    pkcs8::{EncodePrivateKey, LineEnding},
    RsaPrivateKey,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use zxcvbn::zxcvbn;

#[derive(Deserialize, Validate)]
#[validate(schema(function = "validate_password_stength", skip_on_field_errors = false))]
pub struct RegisterForm {
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}

fn validate_password_stength(form: &RegisterForm) -> Result<(), ValidationError> {
    let entropy = zxcvbn(&form.password, &[&form.email, &form.username]).map_err(|err| {
        let mut error = ValidationError::new("PASSWORD_STRENGTH_VALIDATION");
        error.add_param("error".into(), &err.to_string());
        error
    })?;

    if entropy.score() < 3 {
        let mut error = ValidationError::new("PASSWORD_STRENGTH_VALIDATION");
        error.add_param("error".into(), &"Password too weak");
        return Err(error);
    }

    Ok(())
}

pub async fn post(
    Extension(state): Extension<State>,
    Form(register_form): Form<RegisterForm>,
) -> Result<Response> {
    register_form.validate()?;

    // These queries provide a better user experience than just a random 500 error
    // They are also fine from a performance standpoint since both, the username and the email field, are indexed
    let is_username_taken = user::Entity::find()
        .filter(user::Column::Username.eq(register_form.username.as_str()))
        .one(&state.db_conn)
        .await?
        .is_some();
    if is_username_taken {
        return Ok((StatusCode::BAD_REQUEST, "Username already taken").into_response());
    }

    let is_email_used = user::Entity::find()
        .filter(user::Column::Email.eq(register_form.email.as_str()))
        .one(&state.db_conn)
        .await?
        .is_some();
    if is_email_used {
        return Ok((StatusCode::BAD_REQUEST, "Email address already in use").into_response());
    }

    let hashed_password_fut = crate::blocking::cpu(move || {
        let salt = SaltString::generate(rand::thread_rng());
        let argon2 = Argon2::default();

        argon2
            .hash_password(register_form.password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    });
    let private_key_fut =
        crate::blocking::cpu(|| RsaPrivateKey::new(&mut rand::thread_rng(), 4096));

    let (hashed_password, private_key) = tokio::try_join!(hashed_password_fut, private_key_fut)?;
    let private_key = private_key?.to_pkcs8_pem(LineEnding::LF)?;

    let url = format!(
        "https://{}/users/{}",
        state.config.domain, register_form.username
    );
    let inbox_url = format!("{url}/inbox");

    user::Model {
        id: Uuid::new_v4(),
        username: register_form.username,
        email: Some(register_form.email),
        password: Some(hashed_password?),
        domain: None,
        url,
        inbox_url,
        public_key: None,
        private_key: Some(private_key.to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .into_active_model()
    .insert(&state.db_conn)
    .await?;

    Ok(StatusCode::CREATED.into_response())
}
