use async_graphql::Result;

use crate::{models::ResolveMusicLinkInput, utils::get_base_http_client};

pub struct Service;

impl Service {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn resolve_music_link(&self, input: ResolveMusicLinkInput) -> Result<String> {
        tracing::debug!("Received link: {:?}", input);
        let http_client = get_base_http_client(None);
        Ok("https://music.youtube.com/watch?v=dQw4w9WgXcQ".to_string())
    }
}
