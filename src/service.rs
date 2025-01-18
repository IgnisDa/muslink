use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello, world!".to_string()
    }
}
