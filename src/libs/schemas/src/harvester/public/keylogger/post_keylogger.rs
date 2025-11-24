use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, schema};

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::KeyLogger::Req)]
pub struct Req(pub Vec<Key>);

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::KeyLogger::Req::Key)]
pub struct Key {
    pub at: NaiveDateTime,
    pub key_pressed: String,
}
