use crate::graphql::GraphQLService;
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql::{ObjectType, SubscriptionType};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;

pub async fn graphql<
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
>(
    State(app_state): State<GraphQLService<Query, Mutation, Subscription>>,
    graphql_request: GraphQLRequest,
) -> GraphQLResponse {
    let inner_request = graphql_request.into_inner();

    app_state.schema().execute(inner_request).await.into()
}

pub async fn graphiql() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

pub async fn health() -> impl IntoResponse {
    "pong".to_string()
}
