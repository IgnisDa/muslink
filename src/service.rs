use async_graphql::Result;

use crate::utils::get_base_http_client;

pub struct Service;

impl Service {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn resolve_music_link(&self, link: String) -> Result<String> {
        tracing::debug!("Received link: {}", link);
        let http_client = get_base_http_client(None);
        dbg!(http_client);
        Ok("https://music.youtube.com/watch?v=dQw4w9WgXcQ".to_string())
    }
}
