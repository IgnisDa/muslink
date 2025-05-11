use std::{collections::HashSet, sync::Arc};

use convert_case::{Case, Casing};
use regex::Regex;
use schematic::{Config, ConfigLoader, validate::not_empty};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde::Serialize;
use service::{MusicLinkInput, MusicLinkService};
use teloxide::{
    prelude::*,
    types::{ParseMode, ReactionType, User},
    utils::html::{link, user_mention},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod migrations;

#[derive(Serialize, Config)]
#[config(env)]
struct AppConfig {
    #[setting(validate = not_empty, env = "DATABASE_URL")]
    database_url: String,
    #[setting(validate = not_empty, env = "TELOXIDE_TOKEN")]
    teloxide_token: String,
}

async fn process_message(
    text: String,
    _config: &AppConfig,
    user: Option<User>,
) -> Result<String, bool> {
    tracing::debug!("Processing message: {}", text);

    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    let has_url = url_regex.is_match(&text);
    let urls: HashSet<_> = url_regex
        .find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();

    if urls.is_empty() {
        tracing::debug!("No URLs found in message");
        return Err(has_url);
    }

    tracing::debug!("Found {} URLs in message", urls.len());
    let mut response = String::new();
    let music_service = MusicLinkService::new().await;
    tracing::debug!("MusicLinkService initialized");

    for url in urls {
        tracing::debug!("Processing URL: {}", url);
        let service_input = MusicLinkInput {
            link: url.clone(),
            user_country: "US".to_string(),
        };

        let result = match music_service.resolve_music_link(service_input).await {
            Ok(result) => {
                tracing::debug!("Successfully resolved music link, found: {}", result.found);
                result
            }
            Err(e) => {
                tracing::warn!("Failed to resolve music link for {}: {}", url, e);
                continue;
            }
        };

        if result.found > 0 {
            tracing::debug!(
                "Processing {} music platforms",
                result.collected_links.len()
            );
            let platforms: Vec<_> = result
                .collected_links
                .iter()
                .filter_map(|music_link| {
                    let platform = format!("{:?}", music_link.platform).to_case(Case::Title);
                    music_link.data.as_ref().map(|data| {
                        tracing::debug!("Found {} link: {}", platform, data.url);
                        link(&data.url, &platform)
                    })
                })
                .collect();

            if !response.is_empty() {
                response.push_str("\n\n");
            }
            response.push_str(&format!("for {}\n{}", url, platforms.join(", ")));
        } else {
            tracing::debug!("No music platforms found for {}", url);
        }
    }

    if response.is_empty() {
        tracing::debug!("No music links found for any URLs");
        return Err(has_url);
    }

    if let Some(user) = user {
        let username = user
            .mention()
            .unwrap_or_else(|| user_mention(user.id, user.full_name().as_str()));
        tracing::debug!("Adding attribution for user: {}", user.full_name());
        response.push_str(&format!("\n\nPosted by {}", username));
    }

    tracing::debug!("Returning response with {} characters", response.len());
    Ok(response)
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
        |bot: Bot, config: Arc<AppConfig>, db: Arc<DatabaseConnection>, msg: Message| async move {
            let user_name = msg
                .from
                .as_ref()
                .map(|user| user.full_name())
                .unwrap_or_else(|| "Unknown".to_string());
            let chat_id = msg.chat.id;

            tracing::info!("Received message from {} in chat {}", user_name, chat_id);
            let text = msg.text().unwrap_or_default();

            match process_message(text.to_string(), &config, msg.from).await {
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
        .dependencies(dptree::deps![Arc::new(config), Arc::new(db)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    tracing::info!("Telegram bot shutdown complete");
    Ok(())
}
