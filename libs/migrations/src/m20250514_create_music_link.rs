use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum MusicLink {
    Id,
    Table,
    CreatedAt,
    SpotifyLink,
    AppleMusicLink,
    EquivalentLinks,
    YoutubeMusicLink,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MusicLink::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MusicLink::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(ColumnDef::new(MusicLink::SpotifyLink).text())
                    .col(ColumnDef::new(MusicLink::AppleMusicLink).text())
                    .col(
                        ColumnDef::new(MusicLink::EquivalentLinks)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .col(ColumnDef::new(MusicLink::YoutubeMusicLink).text())
                    .col(
                        ColumnDef::new(MusicLink::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_music_link_spotify_link")
                    .table(MusicLink::Table)
                    .col(MusicLink::SpotifyLink)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_music_link_apple_music_link")
                    .table(MusicLink::Table)
                    .col(MusicLink::AppleMusicLink)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_music_link_youtube_music_link")
                    .table(MusicLink::Table)
                    .col(MusicLink::YoutubeMusicLink)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
