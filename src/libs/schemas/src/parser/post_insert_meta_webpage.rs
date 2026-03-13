use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Insert::Req)]
pub struct Req {
    pub pages: Vec<Page>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Insert::Req::Page)]
pub struct Page {
    pub url: Url,
    pub keywords: Vec<String>,
    pub title: String,
    pub search_engine: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Insert::Res)]
pub struct Res {}
