use std::collections::HashMap;

use async_graphql::{Error, Result};
use nest_struct::nest_struct;
use reqwest::{Client, Url};
use rust_iso3166::from_alpha2;
use serde::{Deserialize, Serialize};

use crate::{
    models::ResolveMusicLinkInput,
    utils::{get_base_http_client, SONG_LINK_API_URL},
};

pub struct Service {
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SongLinkPlatform {
    Spotify,
    AppleMusic,
    YoutubeMusic,
    #[serde(untagged)]
    Unknown(String),
}

#[nest_struct]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SongLinkResponse {
    page_url: String,
    entity_unique_id: String,
    entities_by_unique_id: HashMap<
        String,
        nest! {
            id: String,
            platforms: Vec<SongLinkPlatform>,
        },
    >,
}

impl Service {
    pub async fn new() -> Self {
        let client = get_base_http_client(None);
        Self { client }
    }

    pub async fn resolve_music_link(&self, input: ResolveMusicLinkInput) -> Result<String> {
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
            .await?;
        dbg!(response);
        Ok("https://music.youtube.com/watch?v=dQw4w9WgXcQ".to_string())
    }
}
