use apalis::prelude::Error;
use entities::{prelude::TelegramBotMusicShareReaction, telegram_bot_music_share_reaction};
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{ChatCompletionMessage, ChatCompletionRequest, Content, MessageRole},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde::Deserialize;

use crate::AppState;

static RATING_PROMPT: &str = include_str!("rating_prompt.txt");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SentimentResponseMood {
    Neutral,
    Positive,
    Negative,
    Unrelated,
}

#[derive(Debug, Deserialize)]
struct SentimentResponse {
    id: String,
    sentiment: SentimentResponseMood,
}

pub async fn rate_unrated_reactions(state: &AppState) -> Result<(), Error> {
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
    let result = match client.chat_completion(req).await {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Failed to send request to OpenAI: {}", e);
            return Ok(());
        }
    };
    let Some(response_text) = result.choices[0].message.content.as_ref() else {
        tracing::error!("Failed to get response from OpenAI");
        return Ok(());
    };
    let parsed = match serde_json::from_str::<Vec<SentimentResponse>>(response_text) {
        Ok(val) => val,
        Err(e) => {
            tracing::error!("Failed to parse response from OpenAI: {}", e);
            return Ok(());
        }
    };
    dbg!(&parsed);
    Ok(())
}
