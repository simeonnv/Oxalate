use utoipa::{ToSchema, schema};

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::Info::Resources::Req)]
pub struct Req {
    pub ram_usage: f32,
    pub cpu_usage: f32,
    pub net_usage_bytes: u32,
}
