use anyhow::Result;
use chrono::Utc;
use entities::{music_link, prelude::MusicLink};
use reqwest::{Client, Url};
use rust_iso3166::{US, from_alpha2};
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, prelude::Expr,
    sea_query::PgFunc,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

mod models;
mod utils;

use models::providers::{SongLinkPlatform, SongLinkResponse};
pub use models::{MusicLinkData, MusicLinkInput, MusicLinkResponse, MusicPlatform};
use utils::{SONG_LINK_API_URL, get_base_http_client};

pub struct MusicLinkService {
    client: Client,
}

impl MusicLinkService {
    pub async fn new() -> Self {
        let client = get_base_http_client(None);
        Self { client }
    }

    async fn get_music_link_from_db(
        &self,
        link: &String,
        db: &DatabaseConnection,
    ) -> Result<Option<music_link::Model>> {
        let music_link = MusicLink::find()
            .filter(Expr::val(link).eq(PgFunc::any(Expr::col(music_link::Column::AllLinks))))
            .one(db)
            .await?;
        if let Some(music_link) = &music_link {
            let mut active: music_link::ActiveModel = music_link.clone().into();
            active.last_interacted_at = ActiveValue::Set(Utc::now());
            active.update(db).await?;
        }
        Ok(music_link)
    }

    async fn save_music_link_to_db(
        &self,
        original_link: &String,
        db: &DatabaseConnection,
        links: &Vec<MusicLinkData>,
    ) -> Result<Uuid> {
        let spotify_link = links
            .iter()
            .find(|link| link.platform == MusicPlatform::Spotify)
            .and_then(|link| link.link.clone());
        let apple_music_link = links
            .iter()
            .find(|link| link.platform == MusicPlatform::AppleMusic)
            .and_then(|link| link.link.clone());
        let youtube_music_link = links
            .iter()
            .find(|link| link.platform == MusicPlatform::YoutubeMusic)
            .and_then(|link| link.link.clone());
        for link in [&spotify_link, &apple_music_link, &youtube_music_link]
            .into_iter()
            .flatten()
        {
            let already = self.get_music_link_from_db(link, db).await?;
            if let Some(already) = already {
                let mut new_links = already.equivalent_links.clone();
                new_links.push(original_link.clone());
                let mut active: music_link::ActiveModel = already.into();
                active.equivalent_links = ActiveValue::Set(new_links);
                active.last_interacted_at = ActiveValue::Set(Utc::now());
                let updated = active.update(db).await?;
                return Ok(updated.id);
            }
        }
        let to_insert = music_link::ActiveModel {
            spotify_link: ActiveValue::Set(spotify_link),
            apple_music_link: ActiveValue::Set(apple_music_link),
            youtube_music_link: ActiveValue::Set(youtube_music_link),
            equivalent_links: ActiveValue::Set(vec![original_link.clone()]),
            ..Default::default()
        };
        let inserted = to_insert.insert(db).await?;
        Ok(inserted.id)
    }

    pub async fn resolve_music_link(
        &self,
        input: MusicLinkInput,
        db: &DatabaseConnection,
    ) -> Result<MusicLinkResponse> {
        tracing::debug!("Received link: {:?}", input);

        let music_link = self.get_music_link_from_db(&input.link, db).await?;
        if let Some(music_link) = music_link {
            tracing::debug!("Found music link in db: {:?}", music_link);
            let mut found = 0;
            let mut collected_links = vec![];
            if let Some(spotify_link) = music_link.spotify_link {
                found += 1;
                collected_links.push(MusicLinkData {
                    link: Some(spotify_link),
                    platform: MusicPlatform::Spotify,
                });
                if let Some(apple_music_link) = music_link.apple_music_link {
                    found += 1;
                    collected_links.push(MusicLinkData {
                        link: Some(apple_music_link),
                        platform: MusicPlatform::AppleMusic,
                    });
                }
                if let Some(youtube_music_link) = music_link.youtube_music_link {
                    found += 1;
                    collected_links.push(MusicLinkData {
                        link: Some(youtube_music_link),
                        platform: MusicPlatform::YoutubeMusic,
                    });
                }
            }
            return Ok(MusicLinkResponse {
                found,
                collected_links,
                id: music_link.id,
            });
        }

        let user_country = from_alpha2(input.user_country.as_str()).unwrap_or(US);

        let url = Url::parse_with_params(
            SONG_LINK_API_URL,
            &[
                ("songIfSingle", "true"),
                ("url", input.link.as_str()),
                ("userCountry", user_country.alpha2),
            ],
        )?;
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<SongLinkResponse>()
            .await
            .ok();

        let mut found = 0;
        let collected_links = MusicPlatform::iter()
            .map(|platform| {
                let sl_platform = match platform {
                    MusicPlatform::Spotify => SongLinkPlatform::Spotify,
                    MusicPlatform::AppleMusic => SongLinkPlatform::AppleMusic,
                    MusicPlatform::YoutubeMusic => SongLinkPlatform::YoutubeMusic,
                };
                let link = response.as_ref().and_then(|resp| {
                    resp.links_by_platform
                        .get(&sl_platform)
                        .map(|link| link.url.clone())
                });
                if link.is_some() {
                    found += 1;
                }
                MusicLinkData { link, platform }
            })
            .collect();

        let id = self
            .save_music_link_to_db(&input.link, db, &collected_links)
            .await?;

        let response = MusicLinkResponse {
            id,
            found,
            collected_links,
        };

        tracing::debug!("Returning response {:?}", response);
        Ok(response)
    }
}
