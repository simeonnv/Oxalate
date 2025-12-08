use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

use chrono::Utc;
use log::info;
use oxalate_urls::{Ipv4UrlRange, Protocol, Urls};
use serde::{Deserialize, Serialize};

use crate::{
    ProxyId,
    scrapper_controller::{ProxyJob, ScraperJobGenerator},
};
const IP_AMOUNT: u32 = 1_000;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Ipv4IteratorJobGenerator {
    pub last_ip: Arc<AtomicU32>,
}

impl ScraperJobGenerator for Ipv4IteratorJobGenerator {
    fn generate_new_job(&self, proxy_id: &ProxyId) -> Arc<ProxyJob> {
        let ip = self.last_ip.fetch_add(IP_AMOUNT, Ordering::Relaxed);
        let ip_range = Ipv4UrlRange {
            from: ip,
            to: ip + IP_AMOUNT,
            index: 0,
            port: None,
            protocol: Protocol::Msp,
        };

        let urls = Urls::Ipv4UrlRange(ip_range);
        info!(
            "generating new job with ip-range: {ip}-{} for {proxy_id}",
            ip + IP_AMOUNT
        );
        Arc::new(ProxyJob {
            urls,
            dead: AtomicBool::new(false),
            assigned_to: proxy_id.to_owned(),
            job_dispatched: Utc::now().naive_utc(),
        })
    }
}
