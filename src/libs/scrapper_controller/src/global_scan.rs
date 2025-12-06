use std::{
    collections::HashMap,
    net::Ipv4Addr,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use chrono::{Duration, Utc};
use dashmap::DashMap;
use log::info;
use oxalate_urls::{Ipv4UrlRange, Protocol, Urls};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{
    Error, ProxyId, save_proxy_outputs,
    scrapper_controller::{ProxyJob, ProxyOutput, ScraperLevel},
};

const IP_AMOUNT: u32 = 65536;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GlobalScan {
    pub active_jobs: DashMap<ProxyId, Arc<ProxyJob>>,
    pub last_ip: Arc<AtomicU32>,
}

impl ScraperLevel for GlobalScan {
    fn req_addresses(&self, proxy_id: &ProxyId) -> Option<Arc<ProxyJob>> {
        if let Some(e) = self.active_jobs.get(proxy_id) {
            info!("sending already allocated job for {proxy_id}!");
            return Some(e.to_owned());
        }

        for mut job in self.active_jobs.iter_mut() {
            let (_, job) = job.pair_mut();
            if job.dead.load(Ordering::Relaxed) {
                let proxy_job = Arc::new(ProxyJob {
                    urls: job.urls.clone(),
                    dead: false.into(),
                    assigned_to: proxy_id.to_owned(),
                    job_dispatched: Utc::now().naive_utc(),
                });
                *job = proxy_job;
                info!("re-distributing a dead job for {proxy_id}!");
                return Some(job.to_owned());
            }
        }

        let ip = self.last_ip.fetch_add(IP_AMOUNT, Ordering::Relaxed);
        let ip_range = Ipv4UrlRange {
            from: ip,
            to: ip + IP_AMOUNT,
            index: 0,
            port: None,
            protocol: Protocol::Https,
        };

        let urls = Urls::Ipv4UrlRange(ip_range);
        info!(
            "generating new job with ip-range: {ip}-{} for {proxy_id}",
            ip + IP_AMOUNT
        );
        let proxy_job = Arc::new(ProxyJob {
            urls,
            dead: AtomicBool::new(false),
            assigned_to: proxy_id.to_owned(),
            job_dispatched: Utc::now().naive_utc(),
        });
        self.active_jobs
            .insert(proxy_id.to_owned(), proxy_job.clone());
        Some(proxy_job)
    }

    async fn complete_job(
        &self,
        proxy_id: &ProxyId,
        proxy_outputs: &[ProxyOutput],
        db_pool: Pool<Postgres>,
    ) -> Result<(), Error> {
        info!("job completed: {proxy_id}");
        let _ = {
            self.active_jobs
                .get(proxy_id)
                .ok_or(Error::DeviceHasNoJob)?
                .value()
                .clone()
        };

        info!("saving proxy outputs in db");
        save_proxy_outputs(proxy_id, proxy_outputs, db_pool).await?;
        self.active_jobs.remove(proxy_id);

        info!("successfully completed proxy job!");
        Ok(())
    }

    fn check_for_dead_jobs(&self) {
        for job in self.active_jobs.iter_mut() {
            let job = job.value();
            let now = Utc::now().naive_utc();
            if !job.dead.load(Ordering::Relaxed)
                && (job.job_dispatched + Duration::minutes(5)) > now
            {
                job.deref().dead.store(true, Ordering::Relaxed);
            }
        }
    }

    fn get_jobs(&self) -> HashMap<ProxyId, Arc<ProxyJob>> {
        self.active_jobs
            .iter()
            .map(|pair| {
                let (k, v) = pair.pair();
                (k.to_owned(), v.to_owned())
            })
            .collect()
    }
}
