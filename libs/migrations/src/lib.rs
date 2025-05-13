pub use sea_orm_migration::prelude::*;

mod m20250511_create_extensions;
mod m20250512_create_telegram_bot_channel;
mod m20250513_create_telegram_bot_user;
mod m20250514_create_music_link;
mod m20250515_create_telegram_bot_music_share;
mod m20250516_create_telegram_bot_music_share_reaction;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250511_create_extensions::Migration),
            Box::new(m20250512_create_telegram_bot_channel::Migration),
            Box::new(m20250513_create_telegram_bot_user::Migration),
            Box::new(m20250514_create_music_link::Migration),
            Box::new(m20250515_create_telegram_bot_music_share::Migration),
            Box::new(m20250516_create_telegram_bot_music_share_reaction::Migration),
        ]
    }
}
