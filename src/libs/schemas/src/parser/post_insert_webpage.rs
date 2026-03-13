use std::collections::HashMap;

use oxalate_scraper_controller::ProxyId;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::ParseAndInsert::Req)]
pub struct Req {
    pub pages: Vec<Page>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::ParseAndInsert::Req::Page)]
pub struct Page {
    pub url: Url,
    pub raw_html: String,
    pub headers: Option<HashMap<String, String>>,
    pub proxy_id: ProxyId,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::ParseAndInsert::Res)]
pub struct Res {}
