// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

use oxalate_scraper_controller::ProxyId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Get::Metric::ConnectedProxies::Res)]
pub struct Res {
    pub connected_proxies: Vec<ProxyId>,
}
