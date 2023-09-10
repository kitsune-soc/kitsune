use self::{mutation::RootMutation, query::RootQuery};
use super::extractor::{AuthExtractor, UserData};
use crate::state::AppState;
use async_graphql::{
    extensions::Tracing,
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, Error, Result, Schema,
};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::{
    debug_handler,
    response::Html,
    routing::{any, get},
    Extension, Router,
};

type GraphQLSchema = Schema<RootQuery, RootMutation, EmptySubscription>;

mod mutation;
mod query;
mod types;

pub trait ContextExt {
    fn state(&self) -> &AppState;
    fn user_data(&self) -> Result<&UserData>;
}

impl ContextExt for &Context<'_> {
    fn state(&self) -> &AppState {
        self.data().expect("[Bug] State missing in GraphQL context")
    }

    fn user_data(&self) -> Result<&UserData> {
        self.data_opt()
            .ok_or_else(|| Error::new("Authentication required"))
    }
}

#[debug_handler(state = AppState)]
async fn graphql_route(
    Extension(schema): Extension<GraphQLSchema>,
    user_data: Option<AuthExtractor<true>>,
    req: GraphQLBatchRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(user_data) = user_data {
        req = req.data(user_data.0);
    }

    schema.execute_batch(req).await.into()
}

#[allow(clippy::unused_async)]
async fn graphiql_route() -> Html<String> {
    let config = GraphQLPlaygroundConfig::new("/graphql")
        .title(concat!(env!("CARGO_PKG_NAME"), " - GraphiQL"));

    Html(playground_source(config))
}

pub fn routes(state: AppState) -> Router<AppState> {
    let schema: GraphQLSchema = Schema::build(
        RootQuery::default(),
        RootMutation::default(),
        EmptySubscription,
    )
    .data(state)
    .extension(Tracing)
    .finish();

    Router::new()
        .route("/graphql", any(graphql_route))
        .route("/graphiql", get(graphiql_route))
        .layer(Extension(schema))
}
