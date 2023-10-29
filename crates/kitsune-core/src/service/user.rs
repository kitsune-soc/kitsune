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
use diesel_async::RunQueryDsl;
use futures_util::future::OptionFuture;
use garde::Validate;
use iso8601_timestamp::Timestamp;
use kitsune_captcha::ChallengeStatus;
use kitsune_db::{
    model::{
        account::{ActorType, NewAccount},
        preference::Preferences,
        user::{NewUser, User},
    },
    schema::{accounts, accounts_preferences, users},
    PgPool,
};
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey,
};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use std::fmt::Write;
use typed_builder::TypedBuilder;
use zxcvbn::zxcvbn;

const MIN_PASSWORD_STRENGTH: u8 = 3;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_strong_password(value: &Option<String>, _context: &()) -> garde::Result {
    let Some(ref value) = value else {
        return Ok(());
    };

    let Ok(entropy) = zxcvbn(value, &[]) else {
        return Err(garde::Error::new("Password strength validation failed"));
    };

    if entropy.score() < MIN_PASSWORD_STRENGTH {
        let feedback_str = entropy.feedback().as_ref().map_or_else(
            || "Password too weak".into(),
            |feedback| {
                let mut feedback_str = String::from('\n');
                for suggestion in feedback.suggestions() {
                    let _ = writeln!(feedback_str, "- {suggestion}");
                }

                if let Some(warning) = feedback.warning() {
                    let _ = write!(feedback_str, "\nWarning: {warning}");
                }

                feedback_str
            },
        );

        return Err(garde::Error::new(feedback_str));
    }

    Ok(())
}

#[derive(Clone, TypedBuilder, Validate)]
pub struct Register {
    /// Username of the new user
    #[garde(length(min = 1, max = 64), pattern(r"[\w\.]+"))]
    username: String,

    /// Email address of the new user
    #[garde(email)]
    email: String,

    /// OIDC ID of the new user
    #[builder(default, setter(strip_option))]
    #[garde(skip)]
    oidc_id: Option<String>,

    /// Password of the new user
    #[builder(default, setter(strip_option))]
    #[garde(custom(is_strong_password))]
    password: Option<String>,

    /// Token required for captcha verification
    #[builder(default)]
    #[garde(skip)]
    captcha_token: Option<String>,

    /// Force the registration to succeed, regardless of closed registrations
    #[builder(setter(strip_bool))]
    #[garde(skip)]
    force_registration: bool,
}

#[derive(Clone, TypedBuilder)]
pub struct UserService {
    db_pool: PgPool,
    job_service: JobService,
    registrations_open: bool,
    url_service: UrlService,
    captcha_service: CaptchaService,
}

impl UserService {
    pub async fn mark_as_confirmed_by_token(&self, confirmation_token: &str) -> Result<()> {
        self.db_pool
            .with_connection(|db_conn| {
                diesel::update(
                    users::table
                        .filter(users::confirmation_token.eq(confirmation_token))
                        .filter(users::confirmed_at.is_null()),
                )
                .set(users::confirmed_at.eq(Timestamp::now_utc()))
                .execute(db_conn)
                .scoped()
            })
            .await?;

        Ok(())
    }

    pub async fn mark_as_confirmed(&self, user_id: Uuid) -> Result<()> {
        self.db_pool
            .with_connection(|db_conn| {
                diesel::update(users::table.find(user_id))
                    .set(users::confirmed_at.eq(Timestamp::now_utc()))
                    .execute(db_conn)
                    .scoped()
            })
            .await?;

        Ok(())
    }

    pub async fn register(&self, register: Register) -> Result<User> {
        if !self.registrations_open && !register.force_registration {
            return Err(ApiError::RegistrationsClosed.into());
        }

        register.validate(&())?;

        if self.captcha_service.enabled() {
            let token = register.captcha_token.ok_or(ApiError::InvalidCaptcha)?;
            let result = self.captcha_service.verify_token(&token).await?;
            if result != ChallengeStatus::Verified {
                return Err(ApiError::InvalidCaptcha.into());
            }
        }

        let hashed_password_fut = OptionFuture::from(register.password.map(|password| {
            kitsune_blocking::crypto(move || {
                let salt = SaltString::generate(rand::thread_rng());
                let argon2 = Argon2::default();

                argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map(|hash| hash.to_string())
            })
        }));
        let private_key_fut =
            kitsune_blocking::crypto(|| RsaPrivateKey::new(&mut rand::thread_rng(), 4096));

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
            .db_pool
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

                    let preferences_fut = diesel::insert_into(accounts_preferences::table)
                        .values(Preferences {
                            account_id,
                            notify_on_follow: true,
                            notify_on_follow_request: true,
                            notify_on_repost: false,
                            notify_on_favourite: false,
                            notify_on_mention: true,
                            notify_on_post_update: true,
                        })
                        .execute(tx);

                    let (_, user, _) = try_join!(account_fut, user_fut, preferences_fut)?;

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
