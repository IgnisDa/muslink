use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum TelegramBotChannel {
    Id,
    Table,
    CreatedAt,
    TelegramChannelId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TelegramBotChannel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TelegramBotChannel::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        ColumnDef::new(TelegramBotChannel::TelegramChannelId)
                            .big_integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(TelegramBotChannel::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
