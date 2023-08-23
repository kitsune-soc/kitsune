use crate::{
    http::graphql::{
        types::{OAuth2Application, User},
        ContextExt,
    },
    service::{oauth2::CreateApp, user::Register},
};
use async_graphql::{
    Context, CustomValidator, InputValueError, Object, Result, Scalar, ScalarType,
};
use std::fmt::Write;
use zxcvbn::zxcvbn;

const MIN_PASSWORD_STRENGTH: u8 = 3;

/// Custom scalar type to have nicer error messages with the custom validator
pub struct Password(String);

impl Password {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Password {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Password> for String {
    fn from(p: Password) -> Self {
        p.0
    }
}

#[Scalar]
impl ScalarType for Password {
    fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
        match value {
            async_graphql::Value::String(s) => Ok(s.into()),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &async_graphql::Value) -> bool {
        matches!(value, async_graphql::Value::String(..))
    }

    fn to_value(&self) -> async_graphql::Value {
        async_graphql::Value::String(self.0.clone())
    }
}

struct PasswordValidator;

impl CustomValidator<Password> for PasswordValidator {
    fn check(&self, value: &Password) -> Result<(), InputValueError<Password>> {
        let Ok(entropy) = zxcvbn(value.as_str(), &[]) else {
            return Err("Password strength validation failed".into());
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

            return Err(feedback_str.into());
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
    ) -> Result<OAuth2Application> {
        let create_app = CreateApp::builder()
            .name(name)
            .redirect_uris(redirect_uri)
            .build();
        let application = ctx.state().service.oauth2.create_app(create_app).await?;

        Ok(application.into())
    }

    pub async fn register_user(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 1, max_length = 64, regex = r"[\w\.]+"))] username: String,
        #[graphql(validator(email))] email: String,
        #[graphql(secret, validator(custom = "PasswordValidator"))] password: Password,
        captcha_token: Option<String>,
    ) -> Result<User> {
        let state = ctx.state();

        let register = Register::builder()
            .username(username)
            .email(email)
            .password(password.into())
            .captcha_token(captcha_token)
            .build();
        let new_user = state.service.user.register(register).await?;

        Ok(new_user.into())
    }
}
