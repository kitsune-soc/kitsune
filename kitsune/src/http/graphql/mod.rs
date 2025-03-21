use self::{mutation::RootMutation, query::RootQuery};
use super::extractor::{AuthExtractor, UserData};
use crate::state::Zustand;
use async_graphql::{
    Context, EmptySubscription, Error, Result, Schema, extensions::Tracing, http::GraphiQLSource,
};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::{Extension, debug_handler, response::Html};

type GraphQLSchema = Schema<RootQuery, RootMutation, EmptySubscription>;

mod mutation;
mod query;
mod types;

pub trait ContextExt {
    fn state(&self) -> &Zustand;
    fn user_data(&self) -> Result<&UserData>;
}

impl ContextExt for &Context<'_> {
    fn state(&self) -> &Zustand {
        self.data().expect("[Bug] State missing in GraphQL context")
    }

    fn user_data(&self) -> Result<&UserData> {
        self.data_opt()
            .ok_or_else(|| Error::new("Authentication required"))
    }
}

#[debug_handler(state = Zustand)]
pub async fn graphql(
    Extension(schema): Extension<GraphQLSchema>,
    user_data: Option<AuthExtractor>,
    req: GraphQLBatchRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(user_data) = user_data {
        req = req.data(user_data.0);
    }

    schema.execute_batch(req).await.into()
}

#[allow(clippy::unused_async)]
pub async fn explorer() -> Html<String> {
    let source = GraphiQLSource::build()
        .title(concat!(env!("CARGO_PKG_NAME"), " - GraphiQL"))
        .endpoint("/graphql")
        .subscription_endpoint("/graphql/ws")
        .finish();

    Html(source)
}

pub fn schema(state: Zustand) -> GraphQLSchema {
    Schema::build(
        RootQuery::default(),
        RootMutation::default(),
        EmptySubscription,
    )
    .data(state)
    .extension(Tracing)
    .finish()
}
