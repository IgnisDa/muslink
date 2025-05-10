use std::collections::HashSet;
use std::sync::Arc;

use convert_case::{Case, Casing};
use regex::Regex;
use schematic::{Config, ConfigLoader, validate::not_empty};
use service::{MusicLinkInput, MusicLinkService};
use teloxide::{
    prelude::*,
    types::{ParseMode, ReactionType, User},
    utils::html::{link, user_mention},
};

#[derive(Config)]
#[config(env)]
struct AppConfig {
    #[setting(validate = not_empty, env = "TELOXIDE_TOKEN")]
    teloxide_token: String,
}

async fn process_message(
    text: String,
    _config: &AppConfig,
    user: Option<User>,
) -> Result<String, bool> {
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
    let music_service = MusicLinkService::new().await;

    for url in urls {
        let service_input = MusicLinkInput {
            link: url.clone(),
            user_country: "US".to_string(),
        };

        let result = match music_service.resolve_music_link(service_input).await {
            Ok(result) => result,
            Err(_) => continue,
        };

        if result.found > 0 {
            let platforms: Vec<_> = result
                .collected_links
                .iter()
                .filter_map(|music_link| {
                    let platform = format!("{:?}", music_link.platform).to_case(Case::Title);
                    music_link
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

    if let Some(user) = user {
        let username = user
            .mention()
            .unwrap_or_else(|| user_mention(user.id, user.full_name().as_str()));
        response.push_str(&format!("\n\nPosted by {}", username));
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
            match process_message(text.to_string(), &config, msg.from).await {
                Ok(response) => {
                    bot.send_message(msg.chat.id, response)
                        .parse_mode(ParseMode::Html)
                        .await?;
                    bot.delete_message(msg.chat.id, msg.id).await?;
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
