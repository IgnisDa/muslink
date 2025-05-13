use std::sync::Arc;

use apalis::prelude::Error;
use entities::{
    prelude::TelegramBotMusicShareReaction,
    telegram_bot_music_share_reaction::{self, SentimentResponseMood},
};
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{ChatCompletionMessage, ChatCompletionRequest, Content, MessageRole},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;

static RATING_PROMPT: &str = include_str!("rating_prompt.txt");

#[derive(Debug, Deserialize)]
struct LlmResponse<T> {
    response: T,
}

#[derive(Debug, Deserialize)]
struct MusicSentiment {
    id: Uuid,
    sentiment: SentimentResponseMood,
}

type MusicSentimentResponse = LlmResponse<Vec<MusicSentiment>>;

pub async fn rate_unrated_reactions(state: &AppState) -> Result<(), Error> {
    let Ok(unrated) = TelegramBotMusicShareReaction::find()
        .filter(telegram_bot_music_share_reaction::Column::LlmSentimentAnalysis.is_null())
        .order_by_asc(telegram_bot_music_share_reaction::Column::CreatedAt)
        .limit(5)
        .all(&state.db)
        .await
    else {
        return Err(Error::Failed(Arc::new(
            "Failed to fetch unrated reactions".into(),
        )));
    };
    tracing::info!("Found {} unrated reactions", unrated.len());
    let Ok(mut client) = OpenAIClient::builder()
        .with_endpoint("https://generativelanguage.googleapis.com/v1beta/openai")
        .with_api_key(state.config.llm_api_token.clone())
        .build()
    else {
        return Err(Error::Failed(Arc::new(
            "Failed to build OpenAI client".into(),
        )));
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
        "gemini-2.0-flash".to_string(),
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
    let result = match client.chat_completion(req).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::Failed(Arc::new(format!("Error: {e}").into())));
        }
    };
    let Some(response_text) = result.choices[0].message.content.as_ref() else {
        return Err(Error::Failed(Arc::new(
            "Failed to get response from OpenAI".into(),
        )));
    };
    let parsed = match serde_json::from_str::<MusicSentimentResponse>(response_text) {
        Ok(val) => val,
        Err(e) => {
            return Err(Error::Failed(Arc::new(
                format!("Error: {e}, Text: {response_text}").into(),
            )));
        }
    };
    tracing::info!("Parsed: {parsed:#?}");
    Ok(())
}
