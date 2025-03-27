use std::sync::Arc;

use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use schematic::{Config, ConfigLoader, validate::not_empty};
use teloxide::prelude::*;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let config = ConfigLoader::<AppConfig>::new().load()?.config;

    let bot = Bot::new(config.teloxide_token.clone());

    let handler = Update::filter_message().endpoint(
        |bot: Bot, config: Arc<AppConfig>, msg: Message| async move {
            let text = msg.text().unwrap_or_default();
            let resolve_music_link = ResolveMusicLink::build_query(resolve_music_link::Variables {
                input: resolve_music_link::ResolveMusicLinkInput {
                    link: text.into(),
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
                return respond(());
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

            bot.send_message(msg.chat.id, format!("Found links:\n{}", links))
                .await?;

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
