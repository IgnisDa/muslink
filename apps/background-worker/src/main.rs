use std::str::FromStr;

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Data, Error, Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronContext, CronStream, Schedule};
use chrono::Local;
use entities::{prelude::TelegramBotMusicShareReaction, telegram_bot_music_share_reaction};
use migrations::MigratorTrait;
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{ChatCompletionMessage, ChatCompletionRequest, Content, MessageRole},
};
use schematic::{Config, ConfigLoader, validate::not_empty};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use serde::Serialize;
use tokio::join;
use tower::load_shed::LoadShedLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static RATING_PROMPT: &str = include_str!("rating_prompt.txt");

#[derive(Serialize, Clone, Config)]
#[config(env)]
struct AppConfig {
    #[setting(validate = not_empty, env = "DATABASE_URL")]
    database_url: String,
    #[setting(validate = not_empty, env = "OPEN_ROUTER_API_KEY")]
    open_router_api_key: String,
}

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    db: DatabaseConnection,
}

#[derive(Debug, Default)]
struct Reminder;

async fn background_worker_job(
    _job: Reminder,
    state: Data<AppState>,
    ctx: CronContext<Local>,
) -> Result<(), Error> {
    tracing::info!("Performing job at: {}", ctx.get_timestamp());
    rate_unrated_reactions(&state).await?;
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
                .enable_tracing()
                .layer(LoadShedLayer::new())
                .catch_panic()
                .data(state)
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

async fn rate_unrated_reactions(state: &AppState) -> Result<(), Error> {
    let Ok(unrated) = TelegramBotMusicShareReaction::find()
        .filter(telegram_bot_music_share_reaction::Column::LlmSentimentAnalysis.is_null())
        .limit(5)
        .all(&state.db)
        .await
    else {
        tracing::error!("Failed to fetch unrated reactions");
        return Ok(());
    };
    tracing::info!("Found {} unrated reactions", unrated.len());
    let Ok(mut client) = OpenAIClient::builder()
        .with_endpoint("https://openrouter.ai/api/v1")
        .with_api_key(state.config.open_router_api_key.clone())
        .build()
    else {
        tracing::error!("Failed to build OpenAI client");
        return Ok(());
    };
    let input = unrated
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "reaction_text": r.reaction_text,
            })
        })
        .collect::<Vec<_>>();
    let req = ChatCompletionRequest::new(
        "deepseek/deepseek-chat-v3-0324:free".to_string(),
        vec![
            ChatCompletionMessage {
                name: None,
                tool_calls: None,
                tool_call_id: None,
                role: MessageRole::system,
                content: Content::Text(RATING_PROMPT.to_string()),
            },
            ChatCompletionMessage {
                name: None,
                tool_calls: None,
                tool_call_id: None,
                role: MessageRole::user,
                content: Content::Text(serde_json::to_string(&input).unwrap()),
            },
        ],
    );
    let Ok(result) = client.chat_completion(req).await else {
        tracing::error!("Failed to send request to OpenAI");
        return Ok(());
    };
    dbg!(&result);
    Ok(())
}
