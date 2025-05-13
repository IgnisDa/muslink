use std::collections::HashMap;

use nest_struct::nest_struct;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum MusicPlatform {
    Spotify,
    AppleMusic,
    YoutubeMusic,
}

#[derive(Debug)]
pub struct MusicLinkInput {
    pub link: String,
    pub user_country: String,
}

#[derive(Debug)]
pub struct MusicLinkData {
    pub link: Option<String>,
    pub platform: MusicPlatform,
}

#[derive(Debug)]
pub struct MusicLinkResponse {
    pub id: Uuid,
    pub found: u8,
    pub collected_links: Vec<MusicLinkData>,
}

pub mod providers {
    use super::*;

    #[derive(Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
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
                pub id: String,
                pub platforms: Vec<SongLinkPlatform>,
            },
        >,
        pub links_by_platform: HashMap<
            SongLinkPlatform,
            nest! {
                pub url: String,
            },
        >,
    }
}
