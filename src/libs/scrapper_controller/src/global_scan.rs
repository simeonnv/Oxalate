use std::{
    collections::HashMap,
    net::Ipv4Addr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use chrono::Utc;
use log::info;
use oxalate_urls::urls::{HttpBasedContent, ProxyReq, ProxyReqs};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    ProxyId,
    scrapper_controller::{HttpBasedOutput, ProxyJob, ScraperJobGenerator},
};
const IP_AMOUNT: u32 = 16384;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Ipv4IteratorJobGenerator {
    pub last_ip: Arc<AtomicU32>,
}

impl Default for Ipv4IteratorJobGenerator {
    fn default() -> Self {
        Self {
            last_ip: Arc::new(1409286144.into()),
        }
    }
}

impl ScraperJobGenerator for Ipv4IteratorJobGenerator {
    fn generate_new_job(&self, proxy_id: &ProxyId) -> Arc<ProxyJob> {
        let ip = self.last_ip.fetch_add(IP_AMOUNT, Ordering::Relaxed);

        let mut reqs = vec![];
        for current_ip in ip..(ip + IP_AMOUNT) {
            let ipv4 = Ipv4Addr::from(current_ip);
            let url = Url::parse(&format!("https://{ipv4}:443")).unwrap();

            let http_content = HttpBasedContent {
                url,
                headers: HashMap::new(),
                body: None,
                method: oxalate_urls::urls::HttpMethod::Get,
            };
            reqs.push(ProxyReq::Https(http_content));
        }
        let reqs = ProxyReqs(reqs);

        info!(
            "generating new job with ip-range: {ip}-{} for {proxy_id}",
            ip + IP_AMOUNT
        );
        Arc::new(ProxyJob {
            reqs,
            dead: AtomicBool::new(false),
            assigned_to: proxy_id.to_owned(),
            job_dispatched: Utc::now().naive_utc(),
        })
    }
}
