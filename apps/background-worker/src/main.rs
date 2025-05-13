use std::str::FromStr;

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Data, Error, Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronContext, CronStream, Schedule};
use chrono::Local;
use entities::{prelude::TelegramBotMusicShareReaction, telegram_bot_music_share_reaction};
use migrations::MigratorTrait;
use schematic::{Config, ConfigLoader, validate::not_empty};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};
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

async fn background_worker_job(
    _job: Reminder,
    ctx: CronContext<Local>,
    db: Data<DatabaseConnection>,
) -> Result<(), Error> {
    tracing::info!("Performing job at: {}", ctx.get_timestamp());
    rate_unrated_reactions(&*db).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv()?;

    let args: Vec<String> = std::env::args().collect();

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

    if args.len() > 1 && args[1] == "trigger" {
        tracing::info!("Trigger argument detected, running rate_unrated_reactions and exiting");
        rate_unrated_reactions(&db).await?;
        return Ok(());
    }

    tracing::info!("Starting background worker");

    let worker = Monitor::new()
        .register(
            WorkerBuilder::new("background-worker-job")
                .enable_tracing()
                .layer(LoadShedLayer::new())
                .catch_panic()
                .data(db)
                .backend(CronStream::new_with_timezone(
                    Schedule::from_str("0 * * * * *").unwrap(),
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

async fn rate_unrated_reactions(db: &DatabaseConnection) -> Result<(), Error> {
    let unrated = TelegramBotMusicShareReaction::find()
        .filter(telegram_bot_music_share_reaction::Column::LlmSentimentAnalysis.is_null())
        .all(db)
        .await
        .ok();
    dbg!(&unrated);
    Ok(())
}
