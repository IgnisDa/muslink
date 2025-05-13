use std::{collections::HashSet, sync::Arc};

use convert_case::{Case, Casing};
use entities::{
    prelude::{TelegramBotChannel, TelegramBotMusicShare, TelegramBotUser},
    telegram_bot_channel, telegram_bot_music_share, telegram_bot_music_share_reaction,
    telegram_bot_user,
};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, prelude::Uuid,
};
use services::{MusicLinkInput, MusicLinkService};
use teloxide::{
    types::{Message, MessageReactionUpdated},
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
            telegram_channel_id: ActiveValue::Set(telegram_channel_id),
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
        telegram_user_id: ActiveValue::Set(user_id),
        telegram_bot_channel_id: ActiveValue::Set(channel.id),
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

fn get_regex_for_url() -> Regex {
    Regex::new(r"https?://[^\s]+").unwrap()
}

pub fn has_url_in_message(message: Message) -> bool {
    let url_regex = get_regex_for_url();
    url_regex.find(message.text().unwrap_or_default()).is_some()
}

pub fn is_reply_to_message(message: Message) -> bool {
    message.reply_to_message().is_some()
}

pub async fn process_music_share(
    text: String,
    msg: &Message,
    db: Arc<DatabaseConnection>,
) -> Result<ProcessMessageResponse, DbErr> {
    tracing::debug!("Processing message: {}", text);

    let urls: HashSet<_> = get_regex_for_url()
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
    db: &DatabaseConnection,
    sent_message: &Message,
    music_link_ids: Vec<Uuid>,
    received_message: &Message,
) -> Result<(), DbErr> {
    let Some(user) = &received_message.from else {
        tracing::warn!("No user found in message");
        return Ok(());
    };
    tracing::debug!("Processing music link ids: {:?}", music_link_ids);
    let user = find_or_create_telegram_user(
        user.id.0.try_into().unwrap(),
        db,
        received_message.chat.id.0,
    )
    .await?;
    for music_link_id in music_link_ids {
        let to_insert = telegram_bot_music_share::ActiveModel {
            music_link_id: ActiveValue::Set(music_link_id),
            telegram_bot_user_id: ActiveValue::Set(user.id),
            sent_telegram_message_id: ActiveValue::Set(sent_message.id.0.try_into().unwrap()),
            received_telegram_message_id: ActiveValue::Set(
                received_message.id.0.try_into().unwrap(),
            ),
            ..Default::default()
        };
        to_insert.insert(db).await?;
    }
    Ok(())
}

async fn process_reaction(
    text: String,
    db: &DatabaseConnection,
    telegram_channel_id: i64,
    reply_to_message_id: i32,
    reaction_text_message_id: Option<i64>,
) -> Result<(), DbErr> {
    if text.is_empty() {
        tracing::warn!("No text found in reaction");
        return Ok(());
    }
    let linked_shares = TelegramBotMusicShare::find()
        .filter(telegram_bot_music_share::Column::SentTelegramMessageId.eq(reply_to_message_id))
        .all(db)
        .await?;
    for share in linked_shares {
        let to_insert = telegram_bot_music_share_reaction::ActiveModel {
            reaction_text: ActiveValue::Set(text.clone()),
            telegram_bot_music_share_id: ActiveValue::Set(share.id),
            telegram_message_id: ActiveValue::Set(reaction_text_message_id),
            ..Default::default()
        };
        to_insert.insert(db).await?;
    }
    Ok(())
}

pub async fn process_text_reaction(
    message: &Message,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let Some(reply_to_message) = message.reply_to_message() else {
        tracing::warn!("No reply to message found");
        return Ok(());
    };
    process_reaction(
        message.text().unwrap_or_default().to_string(),
        db,
        message.chat.id.0,
        reply_to_message.id.0,
        Some(message.id.0.try_into().unwrap()),
    )
    .await?;
    Ok(())
}

pub async fn process_emoji_reaction(
    db: &DatabaseConnection,
    reaction: &MessageReactionUpdated,
) -> Result<(), DbErr> {
    let new_reaction = reaction
        .new_reaction
        .iter()
        .filter_map(|t| t.emoji().cloned())
        .collect::<Vec<String>>()
        .join(",");
    process_reaction(
        new_reaction,
        db,
        reaction.chat.id.0,
        reaction.message_id.0,
        None,
    )
    .await?;
    Ok(())
}
