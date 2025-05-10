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
        // Convert from GraphQL model to service model
        let service_input = service::MusicLinkInput {
            link: input.link,
            user_country: input.user_country,
        };

        // Call the service function
        let result = self.link_service.resolve_music_link(service_input).await?;

        // Convert back to GraphQL model
        let response = crate::models::convert_to_graphql_response(result);

        tracing::debug!("Returning GraphQL response");
        Ok(response)
    }
}
