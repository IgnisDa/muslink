use std::str::FromStr;

use apalis::{
    layers::{WorkerBuilderExt, retry::RetryPolicy},
    prelude::{Data, Error, Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronContext, CronStream, Schedule};
use chrono::Local;
use functions::rate_unrated_reactions;
use migrations::MigratorTrait;
use schematic::{Config, ConfigLoader, validate::not_empty};
use sea_orm::{Database, DatabaseConnection};
use serde::Serialize;
use tokio::join;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod functions;

#[derive(Serialize, Clone, Config)]
#[config(env)]
struct AppConfig {
    #[setting(validate = not_empty, env = "DATABASE_URL")]
    database_url: String,
    #[setting(validate = not_empty, env = "LLM_API_TOKEN")]
    llm_api_token: String,
}

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    db: DatabaseConnection,
}

#[derive(Debug, Clone, Default)]
struct Reminder;

async fn background_worker_job(
    _job: Reminder,
    state: Data<AppState>,
    ctx: CronContext<Local>,
) -> Result<(), Error> {
    tracing::info!("Performing job at: {}", ctx.get_timestamp());
    rate_unrated_reactions(&state).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv()?;

    let args: Vec<String> = std::env::args().collect();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,apalis::layers::tracing::on_failure=debug",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
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

    let state = AppState { config, db };

    if args.len() > 1 && args[1] == "trigger" {
        tracing::info!("Trigger argument detected, running rate_unrated_reactions and exiting");
        rate_unrated_reactions(&state).await?;
        return Ok(());
    }

    tracing::info!("Starting background worker");

    let worker = Monitor::new()
        .register(
            WorkerBuilder::new("background-worker-job")
                .retry(RetryPolicy::retries(3))
                .enable_tracing()
                .catch_panic()
                .data(state)
                .backend(CronStream::new_with_timezone(
                    Schedule::from_str("1 * * * * *").unwrap(),
                    Local,
                ))
                .build_fn(background_worker_job),
        )
        .run();

    tracing::info!("Worker registered and running");
    let _ = join!(worker);
    tracing::info!("Background worker finished");

    Ok(())
}
