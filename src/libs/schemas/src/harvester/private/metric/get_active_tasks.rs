use std::collections::HashMap;

use oxalate_scraper_controller::{ProxyId, scraper_controller::ActiveProxyTask};
use serde::Serialize;

#[derive(Serialize)]
pub struct Res {
    pub active_tasks: HashMap<ProxyId, ActiveProxyTask>,
}
