use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Search::Req)]
pub struct Req {
    pub text: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Search::Res)]
pub struct Res {
    pub search_results: Vec<SearchResult>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Search::Res::SearchResult)]
pub struct SearchResult {
    pub url: Url,
    pub score: f32,
}
