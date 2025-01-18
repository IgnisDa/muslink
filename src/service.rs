use async_graphql::Result;

pub struct Service;

impl Service {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn resolve_music_link(&self, link: String) -> Result<String> {
        Ok("https://music.youtube.com/watch?v=dQw4w9WgXcQ".to_string())
    }
}
