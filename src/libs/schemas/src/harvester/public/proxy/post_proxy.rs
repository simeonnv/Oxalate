use oxalate_scrapper_controller::scrapper_controller::ProxyOutput;
use oxalate_urls::urls::ProxyReqs;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(as = Post::Proxy::Req)]
pub enum Req {
    RequestUrls,
    ReturnUrlOutputs(Vec<ProxyOutput>),
}

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::Proxy::Res)]
pub struct Res(pub Option<ProxyReqs>);
