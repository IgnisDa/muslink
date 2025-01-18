use async_graphql::{Error, Result};
use reqwest::{Client, Url};
use rust_iso3166::from_alpha2;

use crate::{
    models::ResolveMusicLinkInput,
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

    pub async fn resolve_music_link(&self, input: ResolveMusicLinkInput) -> Result<String> {
        tracing::debug!("Received link: {:?}", input);

        from_alpha2(input.user_country.as_str())
            .ok_or_else(|| Error::new("Invalid country code"))?;

        let url = Url::parse_with_params(
            SONG_LINK_API_URL,
            &[
                ("url", input.link.as_str()),
                ("userCountry", input.user_country.as_str()),
                ("songIfSingle", "true"),
            ],
        )?;
        let response = self.client.get(url).send().await?.error_for_status()?;
        dbg!(response.text().await?);
        Ok("https://music.youtube.com/watch?v=dQw4w9WgXcQ".to_string())
    }
}
