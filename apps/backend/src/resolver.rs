use std::sync::Arc;

use async_graphql::{Context, Object, Result};

use crate::{models::graphql::ResolveMusicLinkInput, service::Service};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn resolve_music_link(
        &self,
        gql_ctx: &Context<'_>,
        input: ResolveMusicLinkInput,
    ) -> Result<String> {
        let service = gql_ctx.data_unchecked::<Arc<Service>>();
        service.resolve_music_link(input).await
    }
}
