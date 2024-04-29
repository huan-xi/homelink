use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use axum::response::{Html, IntoResponse};

pub mod handler;
pub mod query_root;
pub(crate) mod query;


