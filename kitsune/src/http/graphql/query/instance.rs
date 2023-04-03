use async_graphql::{Context, Object, Result, SimpleObject};

use crate::http::graphql::ContextExt;

#[derive(SimpleObject)]
pub struct Instance {
    domain: String,
    version: &'static str,
}

#[derive(Default)]
pub struct InstanceQuery;

#[Object]
impl InstanceQuery {
    #[allow(clippy::unused_async)]
    pub async fn instance(&self, ctx: &Context<'_>) -> Result<Instance> {
        let state = ctx.state();
        let domain = state.service.url.domain().into();

        Ok(Instance {
            domain,
            version: env!("CARGO_PKG_VERSION"),
        })
    }
}
