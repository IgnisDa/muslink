use std::sync::Arc;

use schematic::{Config, ConfigLoader, validate::not_empty};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde::Serialize;
use services::process_message;
use teloxide::{
    prelude::*,
    types::{ParseMode, ReactionType},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod entities;
mod migrations;
mod services;

#[derive(Serialize, Config)]
#[config(env)]
struct AppConfig {
    #[setting(validate = not_empty, env = "DATABASE_URL")]
    database_url: String,
    #[setting(validate = not_empty, env = "TELOXIDE_TOKEN")]
    teloxide_token: String,
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

    tracing::info!("Starting Muslink Telegram Bot");

    let config = ConfigLoader::<AppConfig>::new().load()?.config;
    tracing::info!("Configuration loaded successfully");

    tracing::info!("Connecting to database...");
    let db = Database::connect(&config.database_url).await?;
    tracing::info!("Database connection established");

    tracing::info!("Running database migrations...");
    migrations::Migrator::up(&db, None).await?;
    tracing::info!("Database migrations completed");

    let bot = Bot::new(config.teloxide_token.clone());

    let handler = Update::filter_message().endpoint(
        |bot: Bot, db: Arc<DatabaseConnection>, msg: Message| async move {
            let user_name = msg
                .from
                .as_ref()
                .map(|user| user.full_name())
                .unwrap_or_else(|| "Unknown".to_string());
            let chat_id = msg.chat.id;

            tracing::info!("Received message from {} in chat {}", user_name, chat_id);
            let text = msg.text().unwrap_or_default();

            match process_message(text.to_string(), &msg, db.clone()).await {
                Ok(response) => {
                    tracing::info!("Sending music link response to chat {}", chat_id);
                    bot.send_message(msg.chat.id, response)
                        .parse_mode(ParseMode::Html)
                        .await?;
                    tracing::debug!("Deleting original message");
                    bot.delete_message(msg.chat.id, msg.id).await?;
                }
                Err(has_url) if has_url => {
                    tracing::debug!(
                        "URL detected but no music links found, reacting with sad emoji"
                    );
                    bot.set_message_reaction(msg.chat.id, msg.id)
                        .reaction(vec![ReactionType::Emoji {
                            emoji: "😢".to_string(),
                        }])
                        .await?;
                }
                _ => {
                    tracing::debug!("No URLs detected in message, ignoring");
                }
            }
            respond(())
        },
    );

    tracing::info!("Starting Telegram bot dispatcher");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(db)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    tracing::info!("Telegram bot shutdown complete");
    Ok(())
}
