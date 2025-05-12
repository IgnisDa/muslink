use std::{collections::HashSet, sync::Arc};

use convert_case::{Case, Casing};
use entities::{
    prelude::{TelegramBotChannel, TelegramBotUser},
    telegram_bot_channel, telegram_bot_user,
};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
    prelude::Uuid,
};
use services::{MusicLinkInput, MusicLinkService};
use teloxide::{
    types::Message,
    utils::html::{link, user_mention},
};

async fn find_or_create_telegram_user(
    user_id: i64,
    db: &DatabaseConnection,
    telegram_channel_id: i64,
) -> Result<telegram_bot_user::Model, DbErr> {
    let channel = 'chan: {
        let existing_channel = TelegramBotChannel::find()
            .filter(telegram_bot_channel::Column::TelegramChannelId.eq(telegram_channel_id))
            .one(db)
            .await?;
        if let Some(channel) = existing_channel {
            break 'chan channel;
        }
        let new_channel = telegram_bot_channel::ActiveModel {
            telegram_channel_id: Set(telegram_channel_id),
            ..Default::default()
        };
        new_channel.insert(db).await?
    };
    tracing::debug!("Found or created channel: {}", channel.telegram_channel_id);
    let user = TelegramBotUser::find()
        .filter(telegram_bot_user::Column::TelegramUserId.eq(user_id))
        .filter(telegram_bot_user::Column::TelegramBotChannelId.eq(channel.id))
        .one(db)
        .await?;
    if let Some(user) = user {
        return Ok(user);
    }
    let new_user = telegram_bot_user::ActiveModel {
        telegram_user_id: Set(user_id),
        telegram_bot_channel_id: Set(channel.id),
        ..Default::default()
    };
    let result = new_user.insert(db).await?;
    Ok(result)
}

pub enum ProcessMessageResponse {
    NoUrlDetected,
    HasUrlNoMusicLinksFound,
    HasUrlMusicLinksFound {
        text: String,
        music_link_ids: Vec<Uuid>,
    },
}

pub async fn process_message(
    text: String,
    msg: &Message,
    db: Arc<DatabaseConnection>,
) -> Result<ProcessMessageResponse, DbErr> {
    tracing::debug!("Processing message: {}", text);

    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    let urls: HashSet<_> = url_regex
        .find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();

    if urls.is_empty() {
        tracing::debug!("No URLs found in message");
        return Ok(ProcessMessageResponse::NoUrlDetected);
    }

    tracing::debug!("Found {} URLs in message", urls.len());
    let mut response = String::new();
    let music_service = MusicLinkService::new().await;
    tracing::debug!("MusicLinkService initialized");

    let mut music_link_ids = Vec::new();
    for url in urls {
        tracing::debug!("Processing URL: {}", url);
        let service_input = MusicLinkInput {
            link: url.clone(),
            user_country: "US".to_string(),
        };

        let result = match music_service.resolve_music_link(service_input, &db).await {
            Ok(result) => {
                tracing::debug!("Successfully resolved music link, found: {}", result.found);
                result
            }
            Err(e) => {
                tracing::warn!("Failed to resolve music link for {}: {}", url, e);
                continue;
            }
        };

        music_link_ids.push(result.id);

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
                    music_link.link.as_ref().map(|found_link| {
                        tracing::debug!("Found {} link: {}", platform, found_link);
                        link(found_link, &platform)
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
        return Ok(ProcessMessageResponse::HasUrlNoMusicLinksFound);
    }

    if let Some(user) = &msg.from {
        find_or_create_telegram_user(user.id.0.try_into().unwrap(), &db, msg.chat.id.0).await?;
        let username = user
            .mention()
            .unwrap_or_else(|| user_mention(user.id, user.full_name().as_str()));
        tracing::debug!("Adding attribution for user: {}", user.full_name());
        response.push_str(&format!("\n\nPosted by {}", username));
    }

    tracing::debug!("Returning response with {} characters", response.len());
    Ok(ProcessMessageResponse::HasUrlMusicLinksFound {
        music_link_ids,
        text: response,
    })
}

pub async fn after_process_message(
    message: &Message,
    db: &DatabaseConnection,
    music_link_ids: Vec<Uuid>,
) -> Result<(), DbErr> {
    Ok(())
}
