use std::sync::Arc;

use graphql_client::{GraphQLQuery, Response};
use regex::Regex;
use reqwest::Client;
use schematic::{Config, ConfigLoader, validate::not_empty};
use teloxide::{
    prelude::*,
    types::{ReactionType, ReplyParameters},
};

#[derive(Config)]
#[config(env)]
struct AppConfig {
    #[setting(validate = not_empty, env = "TELOXIDE_TOKEN")]
    teloxide_token: String,
    #[setting(validate = not_empty, env = "MUSLINK_API_BASE_URL")]
    muslink_api_base_url: String,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../libs/generated/backend-schema.graphql",
    query_path = "../../libs/graphql/queries/resolve_music_link.graphql",
    variables_derives = "Debug, Default",
    response_derives = "Debug"
)]
struct ResolveMusicLink;

async fn process_message(text: String, config: &AppConfig) -> (Option<String>, bool) {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    let has_url = url_regex.is_match(&text);
    let url = url_regex.find(&text);

    let Some(url) = url else {
        return (None, has_url);
    };

    let resolve_music_link = ResolveMusicLink::build_query(resolve_music_link::Variables {
        input: resolve_music_link::ResolveMusicLinkInput {
            link: url.as_str().to_string(),
            ..Default::default()
        },
    });

    let client = Client::new();
    let response = client
        .post(config.muslink_api_base_url.clone())
        .json(&resolve_music_link)
        .send()
        .await
        .unwrap()
        .json::<Response<resolve_music_link::ResponseData>>()
        .await
        .unwrap();

    let data = response
        .data
        .unwrap_or_else(|| resolve_music_link::ResponseData {
            resolve_music_link: resolve_music_link::ResolveMusicLinkResolveMusicLink {
                found: 0,
                collected_links: vec![],
            },
        });

    if data.resolve_music_link.found == 0 {
        return (None, has_url);
    }

    let links = data
        .resolve_music_link
        .collected_links
        .iter()
        .filter_map(|link| {
            link.data
                .as_ref()
                .map(|data| format!("{:?}: {}", link.platform, data.url))
        })
        .collect::<Vec<_>>()
        .join("\n");

    (Some(format!("Found links:\n\n{}", links)), has_url)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let config = ConfigLoader::<AppConfig>::new().load()?.config;

    let bot = Bot::new(config.teloxide_token.clone());

    let handler = Update::filter_message().endpoint(
        |bot: Bot, config: Arc<AppConfig>, msg: Message| async move {
            let text = msg.text().unwrap_or_default();
            let (response, has_url) = process_message(text.to_string(), &config).await;

            if let Some(response) = response {
                bot.send_message(msg.chat.id, response)
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            } else if has_url {
                bot.set_message_reaction(msg.chat.id, msg.id)
                    .reaction(vec![ReactionType::Emoji {
                        emoji: "😢".to_string(),
                    }])
                    .await?;
            }
            respond(())
        },
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(config)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
