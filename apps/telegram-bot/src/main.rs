use graphql_client::GraphQLQuery;
use teloxide::prelude::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../libs/generated/backend-schema.graphql",
    query_path = "../../libs/graphql/queries/resolve_music_link.graphql",
    response_derives = "Debug"
)]
struct ResolveMusicLink;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, "Hello, world!").await?;
        Ok(())
    })
    .await;

    Ok(())
}
