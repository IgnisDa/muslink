use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum TelegramBotMusicShareReaction {
    Table,
    LlmSentimentAnalysis,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TelegramBotMusicShareReaction::Table)
                    .add_column(
                        ColumnDef::new(TelegramBotMusicShareReaction::LlmSentimentAnalysis).text(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-telegram_bot_music_share_reaction-llm_sentiment_analysis")
                    .table(TelegramBotMusicShareReaction::Table)
                    .col(TelegramBotMusicShareReaction::LlmSentimentAnalysis)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
