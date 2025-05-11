use anyhow::Result;
use entities::{music_link, prelude::MusicLink};
use reqwest::{Client, Url};
use rust_iso3166::{US, from_alpha2};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use strum::IntoEnumIterator;

mod models;
mod utils;

use models::providers::{SongLinkPlatform, SongLinkResponse};
pub use models::{
    MusicLinkData, MusicLinkInput, MusicLinkPlatformData, MusicLinkResponse, MusicPlatform,
};
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
            .filter(
                music_link::Column::SpotifyLink
                    .eq(link)
                    .or(music_link::Column::AppleMusicLink.eq(link))
                    .or(music_link::Column::YoutubeMusicLink.eq(link)),
            )
            .one(db)
            .await?;
        Ok(music_link)
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
                    platform: MusicPlatform::Spotify,
                    data: Some(MusicLinkPlatformData {
                        id: music_link.id,
                        url: spotify_link,
                    }),
                });
                if let Some(apple_music_link) = music_link.apple_music_link {
                    found += 1;
                    collected_links.push(MusicLinkData {
                        platform: MusicPlatform::AppleMusic,
                        data: Some(MusicLinkPlatformData {
                            id: music_link.id,
                            url: apple_music_link,
                        }),
                    });
                }
                if let Some(youtube_music_link) = music_link.youtube_music_link {
                    found += 1;
                    collected_links.push(MusicLinkData {
                        platform: MusicPlatform::YoutubeMusic,
                        data: Some(MusicLinkPlatformData {
                            id: music_link.id,
                            url: youtube_music_link,
                        }),
                    });
                }
            }
            return Ok(MusicLinkResponse {
                found,
                collected_links,
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
                let platform_id = response.as_ref().and_then(|resp| {
                    resp.entities_by_unique_id
                        .values()
                        .find(|entity| entity.platforms.contains(&sl_platform))
                        .map(|entity| entity.id.clone())
                });
                let url = response.as_ref().and_then(|resp| {
                    resp.links_by_platform
                        .get(&sl_platform)
                        .map(|link| link.url.clone())
                });
                let data =
                    url.and_then(|u| platform_id.map(|id| MusicLinkPlatformData { id, url: u }));
                if data.is_some() {
                    found += 1;
                }
                MusicLinkData { platform, data }
            })
            .collect();

        let response = MusicLinkResponse {
            found,
            collected_links,
        };

        tracing::debug!("Returning response {:?}", response);
        Ok(response)
    }
}
