use async_graphql::{Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

pub mod graphql {
    use super::*;

    #[derive(InputObject, Debug)]
    pub struct ResolveMusicLinkInput {
        pub link: String,
        #[graphql(default = "US")]
        pub user_country: String,
    }

    #[derive(Debug, Serialize, Deserialize, Enum, Clone, Copy, PartialEq, Eq, EnumIter)]
    pub enum ResolveMusicLinkResponseLinkPlatform {
        Spotify,
        AppleMusic,
        YoutubeMusic,
    }

    #[derive(SimpleObject, Debug)]
    pub struct ResolveMusicLinkResponseLinkPlatformData {
        pub id: String,
        pub url: String,
    }

    #[derive(SimpleObject, Debug)]
    pub struct ResolveMusicLinkResponseLink {
        pub platform: ResolveMusicLinkResponseLinkPlatform,
        pub data: Option<ResolveMusicLinkResponseLinkPlatformData>,
    }

    #[derive(SimpleObject, Debug)]
    pub struct ResolveMusicLinkResponse {
        pub found: u8,
        pub collected_links: Vec<ResolveMusicLinkResponseLink>,
    }
}

pub fn convert_to_graphql_response(
    service_response: services::MusicLinkResponse,
) -> graphql::ResolveMusicLinkResponse {
    let collected_links = service_response
        .collected_links
        .into_iter()
        .map(|link| {
            let platform = match link.platform {
                services::MusicPlatform::Spotify => {
                    graphql::ResolveMusicLinkResponseLinkPlatform::Spotify
                }
                services::MusicPlatform::AppleMusic => {
                    graphql::ResolveMusicLinkResponseLinkPlatform::AppleMusic
                }
                services::MusicPlatform::YoutubeMusic => {
                    graphql::ResolveMusicLinkResponseLinkPlatform::YoutubeMusic
                }
            };

            let data = link
                .data
                .map(|d| graphql::ResolveMusicLinkResponseLinkPlatformData {
                    id: d.id,
                    url: d.url,
                });

            graphql::ResolveMusicLinkResponseLink { platform, data }
        })
        .collect();

    graphql::ResolveMusicLinkResponse {
        found: service_response.found,
        collected_links,
    }
}
