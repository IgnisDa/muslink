use std::str::FromStr;

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Error, Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronContext, CronStream, Schedule};
use chrono::Local;
use migrations::MigratorTrait;
use schematic::{Config, ConfigLoader, validate::not_empty};
use sea_orm::Database;
use serde::Serialize;
use tokio::join;
use tower::load_shed::LoadShedLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Config)]
#[config(env)]
struct AppConfig {
    #[setting(validate = not_empty, env = "DATABASE_URL")]
    database_url: String,
}

#[derive(Debug, Default)]
struct Reminder;

async fn schedule_job(_job: Reminder, ctx: CronContext<Local>) -> Result<(), Error> {
    tracing::debug!("Starting schedule_job");
    tracing::info!("Performing job {}", ctx.get_timestamp());
    tracing::debug!("Finished schedule_job");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = ConfigLoader::<AppConfig>::new().load()?.config;
    tracing::info!("Configuration loaded successfully");

    tracing::info!("Connecting to database...");
    let db = Database::connect(&config.database_url).await?;
    tracing::info!("Database connection established");

    tracing::info!("Running database migrations...");
    migrations::Migrator::up(&db, None).await?;
    tracing::info!("Database migrations completed");

    tracing::info!("Starting background worker");

    let worker = Monitor::new()
        .register(
            WorkerBuilder::new("background-worker-job")
                .enable_tracing()
                .layer(LoadShedLayer::new())
                .catch_panic()
                .backend(CronStream::new_with_timezone(
                    Schedule::from_str("* * * * * *").unwrap(),
                    Local,
                ))
                .build_fn(schedule_job),
        )
        .run();

    tracing::info!("Worker registered and running");
    let _ = join!(worker);
    tracing::info!("Background worker finished");

    Ok(())
}
