use async_graphql::{Error, Result};
use reqwest::{Client, Url};
use rust_iso3166::from_alpha2;

use crate::{
    models::{
        graphql::{ResolveMusicLinkInput, ResolveMusicLinkResponse},
        providers::SongLinkResponse,
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
            .await?;

        todo!()
    }
}
