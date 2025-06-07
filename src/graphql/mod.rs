mod controller;
mod handler;
pub mod schema;

use crate::graphql::handler::{graphiql, graphql};
use async_graphql::{ObjectType, Schema, SubscriptionType};
use axum::Router;
use axum::http::{HeaderName, Method, Request};
use axum::routing::get;
use axum::serve::Serve;
use handler::health;
use std::env;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, field, info, info_span};

pub struct GraphQLService<
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
> {
    name: &'static str,
    default_port: &'static str,
    schema: Schema<Query, Mutation, Subscription>,
}

impl<
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
> Clone for GraphQLService<Query, Mutation, Subscription>
{
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            default_port: self.default_port,
            schema: self.schema.clone(),
        }
    }
}

impl<
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
> GraphQLService<Query, Mutation, Subscription>
{
    #[must_use]
    pub fn new(
        name: &'static str,
        default_port: &'static str,
        schema: Schema<Query, Mutation, Subscription>,
    ) -> Self {
        Self {
            name,
            default_port,
            schema,
        }
    }

    #[must_use]
    pub fn schema(&self) -> &Schema<Query, Mutation, Subscription> {
        &self.schema
    }

    pub async fn start(&self) -> std::io::Result<Serve<TcpListener, Router, Router>> {
        let port = env::var("PORT").unwrap_or(self.default_port.to_string());

        info!(event = "serverStart", port);

        let cors =
            CorsLayer::permissive().allow_methods([Method::GET, Method::POST, Method::OPTIONS]);

        let x_request_id = HeaderName::from_static("x-request-id");

        let app = Router::new()
            .route(&format!("/{}", self.name), get(graphiql).post(graphql))
            .route("/ping", get(health))
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(move |request: &Request<_>| {
                        let span = info_span!(
                            "http_request",
                            request_id = field::Empty,
                            host = field::Empty,
                            user_agent = field::Empty,
                            accept = field::Empty,
                            origin = field::Empty,
                            referer = field::Empty,
                            accept_encoding = field::Empty,
                            accept_language = field::Empty,
                        );

                        if let Some(host) = request.headers().get("host") {
                            span.record("host", format!("{host:?}"));
                        };
                        if let Some(user_agent) = request.headers().get("user-agent") {
                            span.record("user_agent", format!("{user_agent:?}"));
                        };
                        if let Some(accept) = request.headers().get("accept") {
                            span.record("accept", format!("{accept:?}"));
                        };
                        if let Some(origin) = request.headers().get("origin") {
                            span.record("origin", format!("{origin:?}"));
                        };
                        if let Some(referer) = request.headers().get("referer") {
                            span.record("referer", format!("{referer:?}"));
                        };
                        if let Some(accept_encoding) = request.headers().get("accept-encoding") {
                            span.record("accept_encoding", format!("{accept_encoding:?}"));
                        };
                        if let Some(accept_language) = request.headers().get("accept-language") {
                            span.record("accept_language", format!("{accept_language:?}"));
                        };

                        span
                    })
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .layer(SetRequestIdLayer::new(
                x_request_id.clone(),
                MakeRequestUuid,
            ))
            .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
            .layer(CompressionLayer::new())
            .with_state(self.clone());

        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

        let server = axum::serve(listener, app);

        Ok(server)
    }
}
