use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::entities::telegram_bot_channel;

pub async fn find_or_create_channel(
    db: &DatabaseConnection,
    telegram_channel_id: i64,
) -> Result<telegram_bot_channel::Model, DbErr> {
    let existing_channel = telegram_bot_channel::Entity::find()
        .filter(telegram_bot_channel::Column::TelegramChannelId.eq(telegram_channel_id))
        .one(db)
        .await?;

    if let Some(channel) = existing_channel {
        return Ok(channel);
    }

    let new_channel = telegram_bot_channel::ActiveModel {
        telegram_channel_id: Set(telegram_channel_id),
        ..Default::default()
    };

    let result = new_channel.insert(db).await?;
    Ok(result)
}
