use sea_orm_migration::prelude::*;

use crate::{
    m20250513_create_telegram_bot_user::TelegramBotUser, m20250514_create_music_link::MusicLink,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum TelegramBotMusicShare {
    Id,
    Table,
    CreatedAt,
    MusicLinkId,
    TelegramBotUserId,
    SentTelegramMessageId,
    ReceivedTelegramMessageId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TelegramBotMusicShare::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TelegramBotMusicShare::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShare::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShare::MusicLinkId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShare::TelegramBotUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShare::SentTelegramMessageId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotMusicShare::ReceivedTelegramMessageId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-telegram_bot_music_share-music_link_id")
                            .from(
                                TelegramBotMusicShare::Table,
                                TelegramBotMusicShare::MusicLinkId,
                            )
                            .to(MusicLink::Table, MusicLink::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-telegram_bot_music_share-telegram_bot_user_id")
                            .from(
                                TelegramBotMusicShare::Table,
                                TelegramBotMusicShare::TelegramBotUserId,
                            )
                            .to(TelegramBotUser::Table, TelegramBotUser::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-telegram_bot_music_share-sent_telegram_message_id")
                    .table(TelegramBotMusicShare::Table)
                    .col(TelegramBotMusicShare::SentTelegramMessageId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
