use async_graphql::InputObject;

#[derive(InputObject, Debug)]
pub struct ResolveMusicLinkInput {
    pub link: String,
}
