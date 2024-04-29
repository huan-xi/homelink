use async_graphql::dynamic::Schema;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use log::info;
// use opentelemetry::trace::TraceContextExt;
use serde::Serialize;
use tracing::{Instrument, Level, span};
// use tracing::{info, span, Instrument, Level};
// use tracing_opentelemetry::OpenTelemetrySpanExt;


pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql")
            .subscription_endpoint("/graphql/ws"),
    ))
}

pub async fn graphql_handler(
    schema: Extension<Schema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let span = span!(Level::INFO, "graphql_execution");
    let response = async move {
        schema.execute(req.into_inner()).await
    }.instrument(span.clone()).await;
    response.into()
}