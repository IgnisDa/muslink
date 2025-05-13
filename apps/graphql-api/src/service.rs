use async_graphql::Result;
use sea_orm::DatabaseConnection;

use crate::models::graphql::{ResolveMusicLinkInput, ResolveMusicLinkResponse};

pub struct Service {
    db: DatabaseConnection,
}

impl Service {
    pub async fn new(db: DatabaseConnection) -> Self {
        tracing::debug!("Initializing GraphQL API service");
        Self { db }
    }

    pub async fn resolve_music_link(
        &self,
        input: ResolveMusicLinkInput,
    ) -> Result<ResolveMusicLinkResponse> {
        tracing::info!(
            "Received music link resolution request for URL: {}",
            input.link
        );
        tracing::debug!("User country: {}", input.user_country);

        let link_service = services::MusicLinkService::new().await;

        let service_input = services::MusicLinkInput {
            link: input.link.clone(),
            user_country: input.user_country.clone(),
        };

        tracing::debug!("Calling service to resolve music link");
        let result = match link_service
            .resolve_music_link(service_input, &self.db)
            .await
        {
            Ok(result) => {
                tracing::debug!(
                    "Successfully resolved music link, found {} platforms",
                    result.found
                );
                result
            }
            Err(e) => {
                tracing::warn!("Failed to resolve music link: {}", e);
                return Err(e.into());
            }
        };

        tracing::debug!("Converting service response to GraphQL response");
        let response = crate::models::convert_to_graphql_response(result);

        tracing::info!(
            "Returning GraphQL response for URL: {} with {} platforms",
            input.link,
            response.found
        );
        Ok(response)
    }
}
