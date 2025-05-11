use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum MusicLink {
    Id,
    Table,
    AllLinks,
    CreatedAt,
    SpotifyLink,
    AppleMusicLink,
    EquivalentLinks,
    YoutubeMusicLink,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
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
                    .col(
                        ColumnDef::new(MusicLink::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(MusicLink::EquivalentLinks)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .col(ColumnDef::new(MusicLink::SpotifyLink).text())
                    .col(ColumnDef::new(MusicLink::AppleMusicLink).text())
                    .col(ColumnDef::new(MusicLink::YoutubeMusicLink).text())
                    .col(
                        ColumnDef::new(MusicLink::AllLinks)
                            .not_null()
                            .array(ColumnType::Text),
                    )
                    .to_owned(),
            )
            .await?;
        db.execute_unprepared(
            "
CREATE OR REPLACE FUNCTION update_all_links()
RETURNS TRIGGER AS $$
DECLARE
    temp_links text[];
BEGIN
    temp_links := ARRAY[]::text[];

    IF NEW.spotify_link IS NOT NULL THEN
        temp_links := temp_links || NEW.spotify_link;
    END IF;

    IF NEW.apple_music_link IS NOT NULL THEN
        temp_links := temp_links || NEW.apple_music_link;
    END IF;

    IF NEW.youtube_music_link IS NOT NULL THEN
        temp_links := temp_links || NEW.youtube_music_link;
    END IF;

    IF NEW.equivalent_links IS NOT NULL THEN
        temp_links := temp_links || NEW.equivalent_links;
    END IF;

    NEW.all_links := temp_links;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_all_links_before_insert_or_update
BEFORE INSERT OR UPDATE ON music_link
FOR EACH ROW
EXECUTE FUNCTION update_all_links();
            ",
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
