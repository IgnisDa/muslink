use std::collections::HashSet;
use std::sync::Arc;

use convert_case::{Case, Casing};
use graphql_client::{GraphQLQuery, Response};
use regex::Regex;
use reqwest::Client;
use schematic::{Config, ConfigLoader, validate::not_empty};
use teloxide::{
    prelude::*,
    sugar::request::{RequestLinkPreviewExt, RequestReplyExt},
    types::{ParseMode, ReactionType},
    utils::html::link,
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

async fn process_message(text: String, config: &AppConfig) -> Result<String, bool> {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    let has_url = url_regex.is_match(&text);
    let urls: HashSet<_> = url_regex
        .find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();

    if urls.is_empty() {
        return Err(has_url);
    }

    let mut response = String::new();
    let client = Client::new();

    for url in urls {
        let resolve_music_link = ResolveMusicLink::build_query(resolve_music_link::Variables {
            input: resolve_music_link::ResolveMusicLinkInput {
                link: url.clone(),
                ..Default::default()
            },
        });

        let response_data = client
            .post(config.muslink_api_base_url.clone())
            .json(&resolve_music_link)
            .send()
            .await
            .unwrap()
            .json::<Response<resolve_music_link::ResponseData>>()
            .await
            .unwrap();

        let data = response_data
            .data
            .unwrap_or_else(|| resolve_music_link::ResponseData {
                resolve_music_link: resolve_music_link::ResolveMusicLinkResolveMusicLink {
                    found: 0,
                    collected_links: vec![],
                },
            });

        if data.resolve_music_link.found > 0 {
            let platforms: Vec<_> = data
                .resolve_music_link
                .collected_links
                .iter()
                .filter_map(|api_link| {
                    let platform = format!("{:?}", api_link.platform).to_case(Case::Title);
                    api_link
                        .data
                        .as_ref()
                        .map(|data| link(&data.url, &platform))
                })
                .collect();

            if !response.is_empty() {
                response.push_str("\n\n");
            }
            response.push_str(&format!("for {}\n{}", url, platforms.join(", ")));
        }
    }

    if response.is_empty() {
        return Err(has_url);
    }

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv()?;

    let config = ConfigLoader::<AppConfig>::new().load()?.config;

    let bot = Bot::new(config.teloxide_token.clone());

    let handler = Update::filter_message().endpoint(
        |bot: Bot, config: Arc<AppConfig>, msg: Message| async move {
            let text = msg.text().unwrap_or_default();
            match process_message(text.to_string(), &config).await {
                Ok(response) => {
                    bot.send_message(msg.chat.id, response)
                        .reply_to(msg.id)
                        .parse_mode(ParseMode::Html)
                        .disable_link_preview(true)
                        .await?;
                }
                Err(has_url) if has_url => {
                    bot.set_message_reaction(msg.chat.id, msg.id)
                        .reaction(vec![ReactionType::Emoji {
                            emoji: "😢".to_string(),
                        }])
                        .await?;
                }
                _ => {}
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
