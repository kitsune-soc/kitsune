use crate::http::graphql::{
    types::{OAuth2Application, User},
    ContextExt,
};
use async_graphql::{Context, Object, Result};
use kitsune_core::service::{oauth2::CreateApp, user::Register};

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
        #[graphql(secret)] password: String,
        captcha_token: Option<String>,
    ) -> Result<User> {
        let state = ctx.state();

        let register = Register::builder()
            .username(username)
            .email(email)
            .password(password)
            .captcha_token(captcha_token)
            .build();
        let new_user = state.service.user.register(register).await?;

        Ok(new_user.into())
    }
}
