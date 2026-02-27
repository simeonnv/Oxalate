use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::KeywordGraph::Req)]
pub struct Req {
    pub text: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::KeywordGraph::Res)]
pub struct Res {
    pub nodes: Vec<Node>,
    pub relations: Vec<Relation>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::KeywordGraph::Res::Node)]
pub struct Node {
    pub word: String,
    pub usage: i64,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::KeywordGraph::Res::Relation)]
pub struct Relation {
    pub source_word: String,
    pub weight: i64,
    pub target_word: String,
}
