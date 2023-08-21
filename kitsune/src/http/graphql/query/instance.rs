use crate::{
    consts::VERSION,
    http::graphql::{types::Instance, ContextExt},
};
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct InstanceQuery;

#[Object]
impl InstanceQuery {
    #[allow(clippy::unused_async)]
    pub async fn instance(&self, ctx: &Context<'_>) -> Result<Instance> {
        let state = ctx.state();
        let instance_service = &state.service.instance;
        let url_service = &state.service.url;

        let description = instance_service.description().into();
        let domain = url_service.webfinger_domain().into();
        let local_post_count = instance_service.local_post_count().await?;
        let name = instance_service.name().into();
        let registrations_open = instance_service.registrations_open();
        let user_count = instance_service.user_count().await?;

        Ok(Instance {
            description,
            domain,
            local_post_count,
            name,
            registrations_open,
            user_count,
            version: VERSION,
        })
    }
}
