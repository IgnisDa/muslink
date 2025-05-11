use std::{collections::HashSet, sync::Arc};

use convert_case::{Case, Casing};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use service::{MusicLinkInput, MusicLinkService};
use teloxide::{
    types::User,
    utils::html::{link, user_mention},
};

use crate::{AppConfig, entities::telegram_bot_channel};

pub async fn find_or_create_channel(
    db: &DatabaseConnection,
    telegram_channel_id: i64,
) -> Result<telegram_bot_channel::Model, DbErr> {
    let existing_channel = telegram_bot_channel::Entity::find()
        .filter(telegram_bot_channel::Column::TelegramChannelId.eq(telegram_channel_id))
        .one(db)
        .await?;

    if let Some(channel) = existing_channel {
        return Ok(channel);
    }

    let new_channel = telegram_bot_channel::ActiveModel {
        telegram_channel_id: Set(telegram_channel_id),
        ..Default::default()
    };

    let result = new_channel.insert(db).await?;
    Ok(result)
}

pub async fn process_message(
    text: String,
    user: Option<User>,
    _config: &AppConfig,
    db: Arc<DatabaseConnection>,
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
