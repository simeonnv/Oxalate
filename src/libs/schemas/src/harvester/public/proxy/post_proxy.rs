use oxalate_scrapper_controller::scrapper_controller::ProxyOutput;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[schema(as = Post::Proxy::Req)]
pub enum Req {
    RequestUrls,
    ReturnUrlOutputs(Vec<ProxyOutput>),
}

#[derive(Serialize, ToSchema)]
#[schema(as = Post::Proxy::Res)]
pub struct Res(pub Option<Vec<Url>>);
