use std::collections::HashMap;

use async_graphql::{Enum, InputObject, SimpleObject};
use nest_struct::nest_struct;
use serde::{Deserialize, Serialize};

pub mod graphql {
    use super::*;

    #[derive(InputObject, Debug)]
    pub struct ResolveMusicLinkInput {
        pub link: String,
        #[graphql(default = "US")]
        pub user_country: String,
    }

    #[derive(Debug, Serialize, Deserialize, Enum, Clone, Copy, PartialEq, Eq)]
    pub enum ResolveMusicLinkResponseLinkPlatform {
        Spotify,
        AppleMusic,
        YoutubeMusic,
    }

    #[derive(SimpleObject, Debug)]
    pub struct ResolveMusicLinkResponseLink {
        pub url: Option<String>,
        pub platform: ResolveMusicLinkResponseLinkPlatform,
    }

    #[derive(SimpleObject, Debug)]
    pub struct ResolveMusicLinkResponse {
        pub links: Vec<ResolveMusicLinkResponseLink>,
    }
}

pub mod providers {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum SongLinkPlatform {
        Spotify,
        AppleMusic,
        YoutubeMusic,
        #[serde(untagged)]
        Unknown(String),
    }

    #[nest_struct]
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SongLinkResponse {
        pub page_url: String,
        pub entity_unique_id: String,
        pub entities_by_unique_id: HashMap<
            String,
            nest! {
                id: String,
                platforms: Vec<SongLinkPlatform>,
            },
        >,
    }
}
