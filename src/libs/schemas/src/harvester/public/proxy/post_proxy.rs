use oxalate_scraper_controller::scraper_controller::{ProxyRes, ProxyTask};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Proxy::Req)]
pub enum Req {
    RequestUrls,
    ReturnUrlOutputs(Vec<ProxyRes>),
}

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::Proxy::Res)]
pub struct Res(pub Option<ProxyTask>);
