use async_graphql::{Error, Result};
use reqwest::{Client, Url};
use rust_iso3166::from_alpha2;
use strum::IntoEnumIterator;

use crate::{
    models::{
        graphql::{
            ResolveMusicLinkInput, ResolveMusicLinkResponse, ResolveMusicLinkResponseLink,
            ResolveMusicLinkResponseLinkPlatform, ResolveMusicLinkResponseLinkPlatformData,
        },
        providers::{SongLinkPlatform, SongLinkResponse},
    },
    utils::{get_base_http_client, SONG_LINK_API_URL},
};

pub struct Service {
    client: Client,
}

impl Service {
    pub async fn new() -> Self {
        let client = get_base_http_client(None);
        Self { client }
    }
}

impl Service {
    pub async fn resolve_music_link(
        &self,
        input: ResolveMusicLinkInput,
    ) -> Result<ResolveMusicLinkResponse> {
        tracing::debug!("Received link: {:?}", input);

        from_alpha2(input.user_country.as_str())
            .ok_or_else(|| Error::new("Invalid country code"))?;

        let url = Url::parse_with_params(
            SONG_LINK_API_URL,
            &[
                ("songIfSingle", "true"),
                ("url", input.link.as_str()),
                ("userCountry", input.user_country.as_str()),
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
        let collected_links = ResolveMusicLinkResponseLinkPlatform::iter()
            .map(|platform| {
                let sl_platform = match platform {
                    ResolveMusicLinkResponseLinkPlatform::Spotify => SongLinkPlatform::Spotify,
                    ResolveMusicLinkResponseLinkPlatform::AppleMusic => {
                        SongLinkPlatform::AppleMusic
                    }
                    ResolveMusicLinkResponseLinkPlatform::YoutubeMusic => {
                        SongLinkPlatform::YoutubeMusic
                    }
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
                let data = url.and_then(|u| {
                    platform_id.map(|id| ResolveMusicLinkResponseLinkPlatformData { id, url: u })
                });
                if data.is_some() {
                    found += 1;
                }
                ResolveMusicLinkResponseLink { platform, data }
            })
            .collect();

        let response = ResolveMusicLinkResponse {
            found,
            collected_links,
        };

        tracing::debug!("Returning response {:?}", response);
        Ok(response)
    }
}
