use crate::{
    http::graphql::{
        types::{Oauth2Application, User},
        ContextExt,
    },
    service::{oauth2::CreateApp, user::Register},
};
use async_graphql::{Context, CustomValidator, InputValueError, Object, Result};
use zxcvbn::zxcvbn;

const MIN_PASSWORD_STRENGTH: u8 = 3;

struct PasswordValidator;

impl CustomValidator<String> for PasswordValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
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
    ) -> Result<Oauth2Application> {
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
        #[graphql(secret, validator(custom = "PasswordValidator"))] password: String,
    ) -> Result<User> {
        let state = ctx.state();

        let register = Register::builder()
            .username(username)
            .email(email)
            .password(password)
            .build();
        let new_user = state.service.user.register(register).await?;

        Ok(new_user.into())
    }
}
