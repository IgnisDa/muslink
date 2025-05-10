use std::collections::HashMap;

use nest_struct::nest_struct;
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
pub struct MusicLinkPlatformData {
    pub id: String,
    pub url: String,
}

#[derive(Debug)]
pub struct MusicLink {
    pub platform: MusicPlatform,
    pub data: Option<MusicLinkPlatformData>,
}

#[derive(Debug)]
pub struct MusicLinkResponse {
    pub found: u8,
    pub collected_links: Vec<MusicLink>,
}

// Provider models (SongLink API)
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
