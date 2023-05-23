use super::url::UrlService;
use crate::error::{ApiError, Error, Result};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use futures_util::future::OptionFuture;
use kitsune_db::{
    model::{
        account::{ActorType, NewAccount},
        user::{NewUser, User},
    },
    schema::{accounts, users},
    PgPool,
};
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey,
};
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
    db_conn: PgPool,
    registrations_open: bool,
    url_service: UrlService,
}

impl UserService {
    pub async fn register(&self, register: Register) -> Result<User> {
        if !self.registrations_open {
            return Err(ApiError::RegistrationsClosed.into());
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
        let public_key_id = self.url_service.public_key_id(user_id);

        let mut db_conn = self.db_conn.get().await?;
        let new_user = db_conn
            .transaction(|tx| {
                async move {
                    let account_id = diesel::insert_into(accounts::table)
                        .values(NewAccount {
                            id: user_id,
                            display_name: None,
                            username: register.username.as_str(),
                            locked: false,
                            note: None,
                            local: true,
                            domain: domain.as_str(),
                            actor_type: ActorType::Person,
                            url: url.as_str(),
                            featured_collection_url: None,
                            followers_url: None,
                            following_url: None,
                            inbox_url: None,
                            outbox_url: None,
                            shared_inbox_url: None,
                            public_key_id: public_key_id.as_str(),
                            public_key: public_key_str.as_str(),
                            created_at: None,
                        })
                        .returning(accounts::id)
                        .get_result(tx)
                        .await?;

                    Ok::<_, Error>(
                        diesel::insert_into(users::table)
                            .values(NewUser {
                                id: Uuid::now_v7(),
                                account_id,
                                username: register.username.as_str(),
                                oidc_id: register.oidc_id.as_deref(),
                                email: register.email.as_str(),
                                password: hashed_password.as_deref(),
                                domain: domain.as_str(),
                                private_key: private_key_str.as_str(),
                            })
                            .get_result(tx)
                            .await?,
                    )
                }
                .scope_boxed()
            })
            .await?;

        Ok(new_user)
    }
}
