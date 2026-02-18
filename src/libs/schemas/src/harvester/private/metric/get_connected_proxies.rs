use oxalate_scraper_controller::ProxyId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Get::Metric::ConnectedProxies::Res)]
pub struct Res {
    pub connected_proxies: Vec<ProxyId>,
}
