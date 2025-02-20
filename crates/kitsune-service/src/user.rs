use super::{
    captcha::CaptchaService,
    job::{Enqueue, JobService},
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use diesel_async::RunQueryDsl;
use futures_util::future::OptionFuture;
use garde::Validate;
use kitsune_captcha::ChallengeStatus;
use kitsune_db::{
    PgPool,
    model::{
        account::{ActorType, NewAccount},
        preference::Preferences,
        user::{NewUser, User},
    },
    schema::{accounts, accounts_preferences, users},
    with_transaction,
};
use kitsune_derive::kitsune_service;
use kitsune_error::{Error, ErrorType, Result, bail, kitsune_error};
use kitsune_jobs::mailing::confirmation::SendConfirmationMail;
use kitsune_url::UrlService;
use kitsune_util::{generate_secret, try_join};
use rsa::{
    RsaPrivateKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
};
use speedy_uuid::Uuid;
use std::fmt::Write;
use typed_builder::TypedBuilder;
use zxcvbn::zxcvbn;

const MIN_PASSWORD_STRENGTH: zxcvbn::Score = zxcvbn::Score::Three;

#[inline]
fn conditional_ascii_check(value: &str, ctx: &RegisterContext) -> garde::Result {
    if ctx.allow_non_ascii {
        return Ok(());
    }

    garde::rules::ascii::apply(&value, ())
}

#[allow(clippy::ref_option)] // garde requirement
fn is_strong_password<T>(value: &Option<String>, _context: &T) -> garde::Result {
    let Some(value) = value else {
        return Ok(());
    };

    let entropy = zxcvbn(value, &[]);
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

pub struct RegisterContext {
    allow_non_ascii: bool,
}

#[derive(Clone, TypedBuilder, Validate)]
#[garde(context(RegisterContext))]
pub struct Register {
    /// Username of the new user
    #[garde(
        custom(conditional_ascii_check),
        length(min = 1, max = 64),
        pattern(r"^[\p{L}\p{N}\.]+$")
    )]
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

#[kitsune_service]
pub struct UserService {
    allow_non_ascii_usernames: bool,
    captcha_service: CaptchaService,
    db_pool: PgPool,
    job_service: JobService,
    registrations_open: bool,
    url_service: UrlService,
}

impl UserService {
    #[allow(clippy::too_many_lines)] // TODO: Refactor to get under the limit
    pub async fn register(&self, register: Register) -> Result<User> {
        if !self.registrations_open && !register.force_registration {
            bail!(type = ErrorType::Forbidden.with_body("registrations closed"), "registrations closed");
        }

        register.validate_with(&RegisterContext {
            allow_non_ascii: self.allow_non_ascii_usernames,
        })?;

        if self.captcha_service.enabled() {
            let invalid_captcha = || kitsune_error!(type = ErrorType::BadRequest.with_body("invalid captcha"), "invalid captcha");

            let token = register.captcha_token.ok_or_else(invalid_captcha)?;
            let result = self.captcha_service.verify_token(&token).await?;
            if result != ChallengeStatus::Verified {
                return Err(invalid_captcha());
            }
        }

        let hashed_password_fut = OptionFuture::from(register.password.map(|password| {
            blowocking::crypto(move || {
                let salt = SaltString::generate(rand::thread_rng());
                let argon2 = Argon2::default();

                argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map(|hash| hash.to_string())
            })
        }));
        let private_key_fut =
            blowocking::crypto(|| RsaPrivateKey::new(&mut rand::thread_rng(), 4096));

        let (hashed_password, private_key) = tokio::join!(hashed_password_fut, private_key_fut);
        let hashed_password = hashed_password.transpose()?.transpose()?;

        let private_key = private_key??;
        let public_key_str = private_key.as_ref().to_public_key_pem(LineEnding::LF)?;
        let private_key_str = private_key.to_pkcs8_pem(LineEnding::LF)?;

        let account_id = Uuid::now_v7();
        let domain = self.url_service.domain().to_string();
        let url = self.url_service.user_url(account_id);
        let public_key_id = self.url_service.public_key_id(account_id);

        let new_user = with_transaction!(self.db_pool, |tx| {
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
        })?;

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

#[cfg(test)]
mod test {
    use super::{Register, RegisterContext};
    use garde::Validate;

    #[test]
    fn alphanumeric_username() {
        let valid_usernames = [
            "aumetra",
            "AUMETRA",
            "aum3tr4",
            "äumäträ",
            "아우멭라",
            "あうめtら",
        ];

        for username in valid_usernames {
            let register = Register::builder()
                .email("whatever@kitsune.example".into())
                .password("verysecurepassword123".into())
                .username(username.into())
                .build();

            assert!(
                register
                    .validate_with(&RegisterContext {
                        allow_non_ascii: true
                    })
                    .is_ok(),
                "{username} is considered invalid",
            );
        }

        let invalid_usernames = [",,,", "🎃spooky", "weewoo 🚨"];

        for username in invalid_usernames {
            let register = Register::builder()
                .email("whatever@kitsune.example".into())
                .password("verysecurepassword123".into())
                .username(username.into())
                .build();

            assert!(
                register
                    .validate_with(&RegisterContext {
                        allow_non_ascii: true,
                    })
                    .is_err(),
                "{username} is considered valid",
            );
        }
    }

    #[test]
    fn deny_non_ascii() {
        let register = Register::builder()
            .email("whatever@kitsune.example".into())
            .password("verysecurepassword123".into())
            .username("äumeträ".into())
            .build();

        assert!(
            register
                .validate_with(&RegisterContext {
                    allow_non_ascii: false
                })
                .is_err()
        );
    }
}
