use crate::{
    http::graphql::{
        types::{OAuth2Application, User},
        ContextExt,
    },
    service::{oauth2::CreateApp, user::Register},
};
use async_graphql::{Context, InputValueError, Object, Result, Scalar, ScalarType};

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
        username: String,
        email: String,
        #[graphql(secret)] password: Password,
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
