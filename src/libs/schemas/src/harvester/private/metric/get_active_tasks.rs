use std::collections::HashMap;

use oxalate_scraper_controller::{ProxyId, scraper_controller::ActiveProxyTask};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[schema(as = Get::Metric::ActiveTasks::Res)]
pub struct Res {
    pub active_tasks: HashMap<ProxyId, ActiveProxyTask>,
}
