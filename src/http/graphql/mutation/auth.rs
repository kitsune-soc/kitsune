use crate::{
    db::entity::{oauth::access_token, user},
    http::graphql::ContextExt,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use async_graphql::{Context, CustomValidator, Error, InputObject, Object, Result};
use chrono::Utc;
use rsa::{
    pkcs8::{EncodePrivateKey, LineEnding},
    RsaPrivateKey,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use uuid::Uuid;
use zxcvbn::zxcvbn;

const MIN_PASSWORD_STRENGTH: u8 = 3;

struct PasswordValidator;

impl CustomValidator<String> for PasswordValidator {
    fn check(&self, value: &String) -> Result<(), String> {
        let Ok(entropy) = zxcvbn(value.as_str(), &[]) else {
            return Err("Password strength validation failed".into());
        };

        if entropy.score() < MIN_PASSWORD_STRENGTH {
            return Err("Password too weak".into());
        }

        Ok(())
    }
}

#[derive(InputObject)]
pub struct RegisterData {
    pub username: String,
    #[graphql(validator(email))]
    pub email: String,
    #[graphql(secret, validator(custom = "PasswordValidator"))]
    pub password: String,
}

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    pub async fn login(
        &self,
        ctx: &Context<'_>,
        username: String,
        #[graphql(secret)] password: String,
    ) -> Result<access_token::Model> {
        let state = ctx.state();
        let Some(user) = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .filter(user::Column::Domain.is_null())
            .one(&state.db_conn)
            .await?
        else {
            return Err(Error::new("User not found"));
        };

        let is_valid = crate::blocking::cpu(move || {
            let argon2 = Argon2::default();
            let hashed_password = user.password.unwrap();
            let hashed_password = PasswordHash::new(hashed_password.as_str())?;
            Ok::<_, Error>(
                argon2
                    .verify_password(password.as_bytes(), &hashed_password)
                    .is_ok(),
            )
        })
        .await??;

        if !is_valid {
            return Err(Error::new("Invalid password"));
        }

        let token_data: [u8; 32] = rand::random();
        let token = hex::encode(token_data);

        todo!();

        /* Ok(token::Model {
            token,
            user_id: user.id,
            created_at: Utc::now(),
        }
        .into_active_model()
        .insert(&state.db_conn)
        .await?) */
    }

    pub async fn register(
        &self,
        ctx: &Context<'_>,
        register_data: RegisterData,
    ) -> Result<user::Model> {
        let state = ctx.state();

        // These queries provide a better user experience than just a random 500 error
        // They are also fine from a performance standpoint since both, the username and the email field, are indexed
        let is_username_taken = user::Entity::find()
            .filter(user::Column::Username.eq(register_data.username.as_str()))
            .one(&state.db_conn)
            .await?
            .is_some();
        if is_username_taken {
            return Err(Error::new("Username already taken"));
        }

        let is_email_used = user::Entity::find()
            .filter(user::Column::Email.eq(register_data.email.as_str()))
            .one(&state.db_conn)
            .await?
            .is_some();
        if is_email_used {
            return Err(Error::new("Email already in use"));
        }

        let hashed_password_fut = crate::blocking::cpu(move || {
            let salt = SaltString::generate(rand::thread_rng());
            let argon2 = Argon2::default();

            argon2
                .hash_password(register_data.password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
        });
        let private_key_fut =
            crate::blocking::cpu(|| RsaPrivateKey::new(&mut rand::thread_rng(), 4096));

        let (hashed_password, private_key) =
            tokio::try_join!(hashed_password_fut, private_key_fut)?;
        let private_key = private_key?.to_pkcs8_pem(LineEnding::LF)?;

        let url = format!(
            "https://{}/users/{}",
            state.config.domain, register_data.username
        );
        let inbox_url = format!("{url}/inbox");

        let new_user = user::Model {
            id: Uuid::new_v4(),
            username: register_data.username,
            email: Some(register_data.email),
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

        Ok(new_user)
    }
}
