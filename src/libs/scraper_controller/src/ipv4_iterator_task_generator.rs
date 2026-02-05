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
use oxalate_urls::urls::{HttpBasedContent, MspContent, ProxyReq, ProxyReqs};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    ProxyId,
    scrapper_controller::{HttpBasedOutput, ProxyJob, ScraperJobGenerator},
};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Ipv4IteratorJobGenerator {
    pub last_ip: Arc<AtomicU32>,
}

impl ScraperJobGenerator for Ipv4IteratorJobGenerator {
    fn generate_new_job(&self, proxy_id: &ProxyId, job_size: u32) -> Option<Arc<ProxyJob>> {
        let ip = self.last_ip.fetch_add(job_size, Ordering::Relaxed);

        let mut reqs = vec![];
        for current_ip in ip..(ip + job_size) {
            let ipv4 = Ipv4Addr::from(current_ip);
            let url = Url::parse(&format!("msp://{ipv4}:443")).unwrap();

            let msp_req = MspContent { url };
            reqs.push(ProxyReq::Msp(msp_req));

            // let http_content = HttpBasedContent {
            //     url,
            //     headers: HashMap::new(),
            //     body: None,
            //     method: oxalate_urls::urls::HttpMethod::Get,
            // };
            // reqs.push(ProxyReq::Https(http_content));
        }
        let reqs = ProxyReqs(reqs);

        info!(
            "generating new job with ip-range: {ip}-{} for {proxy_id}",
            ip + job_size
        );
        Some(Arc::new(ProxyJob {
            reqs,
            dead: AtomicBool::new(false),
            assigned_to: proxy_id.to_owned(),
            job_dispatched: Utc::now().naive_utc(),
        }))
    }
}
