use async_graphql::Result;

use crate::models::graphql::{ResolveMusicLinkInput, ResolveMusicLinkResponse};

pub struct Service {
    link_service: service::MusicLinkService,
}

impl Service {
    pub async fn new() -> Self {
        tracing::debug!("Initializing backend service");
        let link_service = service::MusicLinkService::new().await;
        tracing::debug!("MusicLinkService initialized");
        Self { link_service }
    }

    pub async fn resolve_music_link(
        &self,
        input: ResolveMusicLinkInput,
    ) -> Result<ResolveMusicLinkResponse> {
        tracing::info!("Received music link resolution request for URL: {}", input.link);
        tracing::debug!("User country: {}", input.user_country);
        
        let service_input = service::MusicLinkInput {
            link: input.link.clone(),
            user_country: input.user_country.clone(),
        };

        tracing::debug!("Calling service to resolve music link");
        let result = match self.link_service.resolve_music_link(service_input).await {
            Ok(result) => {
                tracing::debug!("Successfully resolved music link, found {} platforms", result.found);
                result
            },
            Err(e) => {
                tracing::warn!("Failed to resolve music link: {}", e);
                return Err(e.into());
            }
        };

        tracing::debug!("Converting service response to GraphQL response");
        let response = crate::models::convert_to_graphql_response(result);

        tracing::info!("Returning GraphQL response for URL: {} with {} platforms", input.link, response.found);
        Ok(response)
    }
}
