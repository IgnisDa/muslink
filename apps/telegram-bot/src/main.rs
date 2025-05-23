use std::sync::Arc;

use functions::{
    ProcessMessageResponse, after_process_message, has_url_in_message, is_reply_to_message,
    process_emoji_reaction, process_music_share, process_text_reaction,
};
use schematic::{Config, ConfigLoader, validate::not_empty};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde::Serialize;
use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    payloads::{SendMessageSetters, SetMessageReactionSetters},
    prelude::{Dispatcher, Requester},
    respond,
    types::{Message, MessageReactionUpdated, ParseMode, ReactionType, Update},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod functions;

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

    let music_share_handler = Update::filter_message()
        .filter(has_url_in_message)
        .endpoint(
            |bot: Bot, msg: Message, db: Arc<DatabaseConnection>| async move {
                let chat_id = msg.chat.id;
                let text = msg.text().unwrap_or_default();

                match process_music_share(text.to_string(), &msg, db.clone()).await {
                    Err(e) => {
                        tracing::error!("Failed to process message: {}", e);
                    }
                    Ok(response) => match response {
                        ProcessMessageResponse::NoUrlDetected => {
                            tracing::debug!("No URL detected in message, ignoring");
                        }
                        ProcessMessageResponse::HasUrlNoMusicLinksFound => {
                            tracing::debug!(
                                "URL detected but no music links found, reacting with sad emoji"
                            );
                            bot.set_message_reaction(msg.chat.id, msg.id)
                                .reaction(vec![ReactionType::Emoji {
                                    emoji: "😢".to_string(),
                                }])
                                .await?;
                        }
                        ProcessMessageResponse::HasUrlMusicLinksFound {
                            text,
                            music_link_ids,
                        } => {
                            tracing::info!("Sending music link response to chat {}", chat_id);
                            let sent = bot
                                .send_message(msg.chat.id, text)
                                .parse_mode(ParseMode::Html)
                                .await?;
                            tracing::debug!("Deleting original message");
                            bot.delete_message(msg.chat.id, msg.id).await?;
                            after_process_message(&db, &sent, music_link_ids, &msg)
                                .await
                                .ok();
                        }
                    },
                };

                respond(())
            },
        );

    let text_reaction_handler = Update::filter_message()
        .filter(is_reply_to_message)
        .endpoint(|msg: Message, db: Arc<DatabaseConnection>| async move {
            process_text_reaction(&msg, &db).await.ok();
            respond(())
        });

    let emoji_reaction_handler = Update::filter_message_reaction_updated().endpoint(
        |reaction: MessageReactionUpdated, db: Arc<DatabaseConnection>| async move {
            process_emoji_reaction(&db, &reaction).await.ok();
            respond(())
        },
    );

    tracing::info!("Starting Telegram bot dispatcher");

    let handler = dptree::entry()
        .branch(music_share_handler)
        .branch(text_reaction_handler)
        .branch(emoji_reaction_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(db)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    tracing::info!("Telegram bot shutdown complete");
    Ok(())
}
