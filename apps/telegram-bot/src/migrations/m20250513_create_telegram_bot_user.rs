use sea_orm_migration::prelude::*;

use super::m20250512_create_telegram_bot_channel::TelegramBotChannel;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum TelegramBotUser {
    Id,
    Table,
    CreatedAt,
    TelegramUserId,
    TelegramBotChannelId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TelegramBotUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TelegramBotUser::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        ColumnDef::new(TelegramBotUser::TelegramUserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotUser::TelegramBotChannelId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotUser::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-telegram_bot_user-channel_id")
                            .from(
                                TelegramBotUser::Table,
                                TelegramBotUser::TelegramBotChannelId,
                            )
                            .to(TelegramBotChannel::Table, TelegramBotChannel::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TelegramBotUser::Table)
                    .name("idx-telegram_bot_user-channel_user_unique")
                    .col(TelegramBotUser::TelegramBotChannelId)
                    .col(TelegramBotUser::TelegramUserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
