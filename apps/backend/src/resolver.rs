use std::sync::Arc;

use async_graphql::{Context, Object, Result};

use crate::{
    models::graphql::{ResolveMusicLinkInput, ResolveMusicLinkResponse},
    service::Service,
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn resolve_music_link(
        &self,
        gql_ctx: &Context<'_>,
        input: ResolveMusicLinkInput,
    ) -> Result<ResolveMusicLinkResponse> {
        tracing::info!("GraphQL resolving music link");
        tracing::debug!("Input URL: {}, Country: {}", input.link, input.user_country);
        
        let service = gql_ctx.data_unchecked::<Arc<Service>>();
        let result = service.resolve_music_link(input).await;
        
        match &result {
            Ok(response) => {
                tracing::info!("GraphQL resolver successfully returned {} platform links", response.found);
            }
            Err(e) => {
                tracing::error!("GraphQL resolver encountered an error: {:?}", e);
            }
        }
        
        result
    }
}
