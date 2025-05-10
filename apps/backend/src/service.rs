use async_graphql::Result;

use crate::models::graphql::{ResolveMusicLinkInput, ResolveMusicLinkResponse};

pub struct Service {
    link_service: service::MusicLinkService,
}

impl Service {
    pub async fn new() -> Self {
        let link_service = service::MusicLinkService::new().await;
        Self { link_service }
    }

    pub async fn resolve_music_link(
        &self,
        input: ResolveMusicLinkInput,
    ) -> Result<ResolveMusicLinkResponse> {
        let service_input = service::MusicLinkInput {
            link: input.link,
            user_country: input.user_country,
        };

        let result = self.link_service.resolve_music_link(service_input).await?;

        let response = crate::models::convert_to_graphql_response(result);

        tracing::debug!("Returning GraphQL response");
        Ok(response)
    }
}
