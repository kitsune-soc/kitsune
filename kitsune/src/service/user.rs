use super::url::UrlService;
use crate::error::{ApiError, Error, Result};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use futures_util::{future::OptionFuture, FutureExt};
use kitsune_db::{
    custom::ActorType,
    entity::{
        accounts,
        prelude::{Accounts, Users},
        users,
    },
};
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    TransactionTrait,
};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, TypedBuilder)]
pub struct Register {
    /// Username of the new user
    username: String,

    /// Email address of the new user
    email: String,

    /// OIDC ID of the new user
    #[builder(default, setter(strip_option))]
    oidc_id: Option<String>,

    /// Password of the new user
    #[builder(default, setter(strip_option))]
    password: Option<String>,
}

#[derive(Clone, TypedBuilder)]
pub struct UserService {
    db_conn: DatabaseConnection,
    registrations_open: bool,
    url_service: UrlService,
}

impl UserService {
    pub async fn register(&self, register: Register) -> Result<users::Model> {
        if !self.registrations_open {
            return Err(ApiError::RegistrationsClosed.into());
        }

        // These queries provide a better user experience than just a random 500 error
        // They are also fine from a performance standpoint since both, the username and the email field, are indexed
        let is_username_taken = Users::find()
            .filter(users::Column::Username.eq(register.username.as_str()))
            .one(&self.db_conn)
            .await?
            .is_some();
        if is_username_taken {
            return Err(ApiError::UsernameTaken.into());
        }

        let is_email_used = Users::find()
            .filter(users::Column::Email.eq(register.email.as_str()))
            .one(&self.db_conn)
            .await?
            .is_some();
        if is_email_used {
            return Err(ApiError::EmailTaken.into());
        }

        let hashed_password_fut = OptionFuture::from(register.password.map(|password| {
            crate::blocking::cpu(move || {
                let salt = SaltString::generate(rand::thread_rng());
                let argon2 = Argon2::default();

                argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map(|hash| hash.to_string())
            })
        }));
        let private_key_fut =
            crate::blocking::cpu(|| RsaPrivateKey::new(&mut rand::thread_rng(), 4096));

        let (hashed_password, private_key) = tokio::join!(hashed_password_fut, private_key_fut);
        let hashed_password = hashed_password.transpose()?.transpose()?;

        let private_key = private_key??;
        let public_key_str = private_key.as_ref().to_public_key_pem(LineEnding::LF)?;
        let private_key_str = private_key.to_pkcs8_pem(LineEnding::LF)?;

        let user_id = Uuid::now_v7();
        let domain = self.url_service.domain().to_string();
        let url = self.url_service.user_url(user_id);
        let public_key_id = format!("{url}#main-key");

        let new_user = self
            .db_conn
            .transaction(|tx| {
                async move {
                    let insert_result = Accounts::insert(
                        accounts::Model {
                            id: user_id,
                            avatar_id: None,
                            header_id: None,
                            display_name: None,
                            username: register.username.clone(),
                            locked: false,
                            note: None,
                            local: true,
                            domain: domain.clone(),
                            actor_type: ActorType::Person,
                            url: None,
                            featured_collection_url: None,
                            followers_url: None,
                            following_url: None,
                            inbox_url: None,
                            outbox_url: None,
                            shared_inbox_url: None,
                            public_key_id,
                            public_key: public_key_str,
                            created_at: OffsetDateTime::now_utc(),
                            updated_at: OffsetDateTime::now_utc(),
                        }
                        .into_active_model(),
                    )
                    .exec(tx)
                    .await?;

                    let new_user = users::Model {
                        id: Uuid::now_v7(),
                        account_id: insert_result.last_insert_id,
                        username: register.username,
                        oidc_id: register.oidc_id,
                        email: register.email,
                        password: hashed_password,
                        domain,
                        private_key: private_key_str.to_string(),
                        created_at: OffsetDateTime::now_utc(),
                        updated_at: OffsetDateTime::now_utc(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    Ok::<_, Error>(new_user)
                }
                .boxed()
            })
            .await?;

        Ok(new_user)
    }
}
