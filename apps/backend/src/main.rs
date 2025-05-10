use std::sync::Arc;

use anyhow::Result;
use async_graphql::{EmptyMutation, EmptySubscription, Schema, http::graphiql_source};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{self, IntoResponse},
    routing::get,
};
use resolver::QueryRoot;
use service::Service;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod models;
mod resolver;
mod service;

async fn graphiql() -> impl IntoResponse {
    response::Html(graphiql_source("/", None))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Muslink Backend API");
    
    tracing::debug!("Initializing service");
    let service = Service::new().await;
    
    tracing::debug!("Building GraphQL schema");
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(Arc::new(service))
        .finish();

    tracing::debug!("Creating API router");
    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));
    tracing::debug!("Router setup complete");

    tracing::debug!("Binding TCP listener");
    let listener = TcpListener::bind("0.0.0.0:5000".to_string()).await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr()?);
    
    tracing::debug!("Starting Axum server");
    let server_result = axum::serve(listener, app).await;
    
    match &server_result {
        Ok(_) => tracing::info!("Server shutdown gracefully"),
        Err(e) => tracing::error!("Server error: {}", e),
    }
    
    server_result.map_err(Into::into)
}
