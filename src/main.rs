use anyhow::Result;
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use resolver::QueryRoot;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod resolver;

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
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

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    let listener = TcpListener::bind(format!("0.0.0.0:5000")).await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
