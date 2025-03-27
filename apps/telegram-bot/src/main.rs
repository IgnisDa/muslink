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
            let resolve_music_link = ResolveMusicLink::build_query(resolve_music_link::Variables {
                input: resolve_music_link::ResolveMusicLinkInput {
                    link: "https://music.youtube.com/watch?v=dTdO8_aWR-g&si=tU4IJLFktsnq_j7I"
                        .into(),
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

            println!("{:?}", response);

            bot.send_message(msg.chat.id, "Hello, world!").await?;

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
