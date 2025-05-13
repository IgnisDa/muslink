use sea_orm_migration::prelude::*;

use crate::{
    m20250513_create_telegram_bot_user::TelegramBotUser,
    m20250515_create_telegram_bot_music_share::TelegramBotMusicShare,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum TelegramBotMusicShareReaction {
    Id,
    Table,
    CreatedAt,
    ReactionText,
    TelegramBotUserId,
    TelegramMessageId,
    TelegramBotMusicShareId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TelegramBotMusicShareReaction::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TelegramBotMusicShareReaction::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShareReaction::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShareReaction::ReactionText)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShareReaction::TelegramBotUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShareReaction::TelegramMessageId)
                            .big_integer(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShareReaction::TelegramBotMusicShareId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-telegram_bot_music_share_reaction-telegram_bot_user_id")
                            .from(
                                TelegramBotMusicShareReaction::Table,
                                TelegramBotMusicShareReaction::TelegramBotUserId,
                            )
                            .to(TelegramBotUser::Table, TelegramBotUser::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-telegram_bot_music_share-music_link_id")
                            .from(
                                TelegramBotMusicShareReaction::Table,
                                TelegramBotMusicShareReaction::TelegramBotMusicShareId,
                            )
                            .to(TelegramBotMusicShare::Table, TelegramBotMusicShare::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-telegram_bot_music_share_reaction-reaction_text")
                    .table(TelegramBotMusicShareReaction::Table)
                    .col(TelegramBotMusicShareReaction::ReactionText)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
