use crate::{
    db::entity::{oauth::application, user},
    http::graphql::ContextExt,
    util::generate_secret,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use async_graphql::{Context, CustomValidator, Error, Object, Result};
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

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    pub async fn register_oauth_application(
        &self,
        ctx: &Context<'_>,
        name: String,
        redirect_uri: String,
    ) -> Result<application::Model> {
        Ok(application::Model {
            id: Uuid::new_v4(),
            secret: generate_secret(),
            name,
            redirect_uri,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
        .into_active_model()
        .insert(&ctx.state().db_conn)
        .await?)
    }

    pub async fn register_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        #[graphql(validator(email))] email: String,
        #[graphql(secret, validator(custom = "PasswordValidator"))] password: String,
    ) -> Result<user::Model> {
        let state = ctx.state();

        // These queries provide a better user experience than just a random 500 error
        // They are also fine from a performance standpoint since both, the username and the email field, are indexed
        let is_username_taken = user::Entity::find()
            .filter(user::Column::Username.eq(username.as_str()))
            .one(&state.db_conn)
            .await?
            .is_some();
        if is_username_taken {
            return Err(Error::new("Username already taken"));
        }

        let is_email_used = user::Entity::find()
            .filter(user::Column::Email.eq(email.as_str()))
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
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
        });
        let private_key_fut =
            crate::blocking::cpu(|| RsaPrivateKey::new(&mut rand::thread_rng(), 4096));

        let (hashed_password, private_key) =
            tokio::try_join!(hashed_password_fut, private_key_fut)?;
        let private_key = private_key?.to_pkcs8_pem(LineEnding::LF)?;

        let url = format!("https://{}/users/{}", state.config.domain, username);
        let inbox_url = format!("{url}/inbox");

        let new_user = user::Model {
            id: Uuid::new_v4(),
            avatar_id: None,
            header_id: None,
            display_name: None,
            note: None,
            username,
            email: Some(email),
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
