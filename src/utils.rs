use std::time::Duration;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT},
    ClientBuilder,
};

pub static USER_AGENT_STR: &str =
    "Muslink (https://github.com/ignisda/muslink) <ignisda2001@gmail.com>";

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
