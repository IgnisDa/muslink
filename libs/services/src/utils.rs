use std::time::Duration;

use reqwest::{
    ClientBuilder,
    header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};

pub static USER_AGENT_STR: &str =
    "Muslink (https://github.com/ignisda/muslink) <ignisda2001@gmail.com>";
pub static SONG_LINK_API_URL: &str = "https://api.song.link/v1-alpha.1/links";

pub fn get_base_http_client(headers: Option<Vec<(HeaderName, HeaderValue)>>) -> reqwest::Client {
    let mut req_headers = HeaderMap::new();
    req_headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    for (header, value) in headers.unwrap_or_default().into_iter() {
        req_headers.insert(header, value);
    }
    ClientBuilder::new()
        .default_headers(req_headers)
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap()
}
