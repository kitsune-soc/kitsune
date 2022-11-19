use self::{mutation::RootMutation, query::RootQuery};
use super::extractor::AuthExtactor;
use crate::{db::entity::user, state::State};
use async_graphql::{http::GraphiQLSource, Context, EmptySubscription, Error, Result, Schema};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::{
    response::Html,
    routing::{any, get},
    Extension, Router,
};

type GraphQLSchema = Schema<RootQuery, RootMutation, EmptySubscription>;

mod mutation;
mod query;

pub trait ContextExt {
    fn state(&self) -> &State;
    fn user(&self) -> Result<&user::Model>;
}

impl ContextExt for &'_ Context<'_> {
    fn state(&self) -> &State {
        self.data().expect("[Bug] State missing in GraphQL context")
    }

    fn user(&self) -> Result<&user::Model> {
        self.data_opt()
            .ok_or_else(|| Error::new("Authentication required"))
    }
}

async fn graphql_route(
    Extension(state): Extension<State>,
    Extension(schema): Extension<GraphQLSchema>,
    AuthExtactor(user): AuthExtactor,
    req: GraphQLBatchRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner().data(state);
    if let Some(user) = user {
        req = req.data(user);
    }

    schema.execute_batch(req).await.into()
}

#[allow(clippy::unused_async)]
async fn graphiql_route() -> Html<String> {
    let page_src = GraphiQLSource::build()
        .title(concat!(env!("CARGO_PKG_NAME"), " - GraphiQL"))
        .endpoint("/graphql")
        .finish();

    Html(page_src)
}

pub fn routes() -> Router {
    let schema: GraphQLSchema = Schema::new(
        RootQuery::default(),
        RootMutation::default(),
        EmptySubscription,
    );

    Router::new()
        .route("/graphql", any(graphql_route))
        .route("/graphiql", get(graphiql_route))
        .layer(Extension(schema))
}
