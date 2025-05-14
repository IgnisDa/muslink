use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "
ALTER TABLE telegram_bot_channel
ADD COLUMN last_interacted_at TIMESTAMP WITH TIME ZONE;

UPDATE telegram_bot_channel SET last_interacted_at = created_at;

ALTER TABLE telegram_bot_channel
ALTER COLUMN last_interacted_at SET NOT NULL;

ALTER TABLE telegram_bot_channel
ALTER COLUMN last_interacted_at SET DEFAULT CURRENT_TIMESTAMP;
        ",
        )
        .await?;

        db.execute_unprepared(
            "
ALTER TABLE telegram_bot_user
ADD COLUMN last_interacted_at TIMESTAMP WITH TIME ZONE;

UPDATE telegram_bot_user SET last_interacted_at = created_at;

ALTER TABLE telegram_bot_user
ALTER COLUMN last_interacted_at SET NOT NULL;

ALTER TABLE telegram_bot_user
ALTER COLUMN last_interacted_at SET DEFAULT CURRENT_TIMESTAMP;
        ",
        )
        .await?;

        db.execute_unprepared(
            "
ALTER TABLE music_link
ADD COLUMN last_interacted_at TIMESTAMP WITH TIME ZONE;

UPDATE music_link SET last_interacted_at = created_at;

ALTER TABLE music_link
ALTER COLUMN last_interacted_at SET NOT NULL;

ALTER TABLE music_link
ALTER COLUMN last_interacted_at SET DEFAULT CURRENT_TIMESTAMP;
        ",
        )
        .await?;
        Ok(())
    }
}
