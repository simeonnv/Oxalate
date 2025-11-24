use utoipa::{ToSchema, schema};

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::Info::Logs::Req)]
pub struct Req {
    pub logs: Vec<Log>,
}

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::Info::Logs::Req::Log)]
pub struct Log {
    pub log_level: String,
    pub body: String,
}
