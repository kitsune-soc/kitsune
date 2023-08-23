use super::{
    captcha::CaptchaService,
    job::{Enqueue, JobService},
    url::UrlService,
};
use crate::{
    error::{ApiError, Error, Result},
    job::mailing::confirmation::SendConfirmationMail,
    try_join,
    util::generate_secret,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use futures_util::future::OptionFuture;
use iso8601_timestamp::Timestamp;
use kitsune_captcha::ChallengeStatus;
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
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

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

    /// Token required for captcha verification
    #[builder(default)]
    captcha_token: Option<String>,
}

#[derive(Clone, TypedBuilder)]
pub struct UserService {
    db_conn: PgPool,
    job_service: JobService,
    registrations_open: bool,
    url_service: UrlService,
    captcha_service: CaptchaService,
}

impl UserService {
    pub async fn mark_as_confirmed_by_token(&self, confirmation_token: &str) -> Result<()> {
        self.db_conn
            .with_connection(|mut db_conn| async move {
                diesel::update(
                    users::table
                        .filter(users::confirmation_token.eq(confirmation_token))
                        .filter(users::confirmed_at.is_null()),
                )
                .set(users::confirmed_at.eq(Timestamp::now_utc()))
                .execute(&mut db_conn)
                .await
                .map_err(Error::from)
            })
            .await?;

        Ok(())
    }

    pub async fn mark_as_confirmed(&self, user_id: Uuid) -> Result<()> {
        self.db_conn
            .with_connection(|mut db_conn| async move {
                diesel::update(users::table.find(user_id))
                    .set(users::confirmed_at.eq(Timestamp::now_utc()))
                    .execute(&mut db_conn)
                    .await
                    .map_err(Error::from)
            })
            .await?;

        Ok(())
    }

    pub async fn register(&self, register: Register) -> Result<User> {
        if !self.registrations_open {
            return Err(ApiError::RegistrationsClosed.into());
        }

        if self.captcha_service.enabled() {
            let token = register.captcha_token.ok_or(ApiError::InvalidCaptcha)?;
            let result = self.captcha_service.verify_token(&token).await?;
            if result != ChallengeStatus::Verified {
                return Err(ApiError::InvalidCaptcha.into());
            }
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

        let account_id = Uuid::now_v7();
        let domain = self.url_service.domain().to_string();
        let url = self.url_service.user_url(account_id);
        let public_key_id = self.url_service.public_key_id(account_id);

        let new_user = self
            .db_conn
            .with_transaction(|tx| {
                async move {
                    let account_fut = diesel::insert_into(accounts::table)
                        .values(NewAccount {
                            id: account_id,
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
                        .execute(tx);

                    let confirmation_token = generate_secret();
                    let user_fut = diesel::insert_into(users::table)
                        .values(NewUser {
                            id: Uuid::now_v7(),
                            account_id,
                            username: register.username.as_str(),
                            oidc_id: register.oidc_id.as_deref(),
                            email: register.email.as_str(),
                            password: hashed_password.as_deref(),
                            domain: domain.as_str(),
                            private_key: private_key_str.as_str(),
                            confirmation_token: confirmation_token.as_str(),
                        })
                        .get_result::<User>(tx);

                    let (_account, user) = try_join!(account_fut, user_fut)?;

                    Ok::<_, Error>(user)
                }
                .scope_boxed()
            })
            .await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(SendConfirmationMail {
                        user_id: new_user.id,
                    })
                    .build(),
            )
            .await?;

        Ok(new_user)
    }

    #[must_use]
    pub fn registrations_open(&self) -> bool {
        self.registrations_open
    }
}
