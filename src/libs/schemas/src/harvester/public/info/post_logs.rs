use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, schema};

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Post::Info::Logs::Req)]
pub struct Req {
    pub logs: Vec<Log>,
}

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Post::Info::Logs::Req::Log)]
pub struct Log {
    pub log_level: String,
    pub time: NaiveDateTime,
    pub file: String,
    pub row: u32,
    pub body: String,
}
