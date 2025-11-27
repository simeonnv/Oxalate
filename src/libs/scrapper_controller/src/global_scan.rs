use std::{
    collections::HashSet,
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
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
// use sqlx::{Pool, Postgres};
use url::Url;

use crate::{
    save_proxy_outputs,
    scrapper_controller::{Error, ProxyJob, ProxyOutput, ScraperLevel},
};

const IP_AMOUNT: u32 = 65536;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct GlobalScan {
    pub active_jobs: DashMap<String, Arc<ProxyJob>>,
    pub last_ip: Arc<AtomicU32>,
}

impl ScraperLevel for GlobalScan {
    fn req_addresses(&self, proxy_id: &str) -> Option<Arc<ProxyJob>> {
        if let Some(e) = self.active_jobs.get(proxy_id) {
            info!("sending already allocated job for {proxy_id}!");
            return Some(e.to_owned());
        }

        for mut job in self.active_jobs.iter_mut() {
            let (_, job) = job.pair_mut();
            if job.dead.load(Ordering::Relaxed) {
                let proxy_job = Arc::new(ProxyJob {
                    urls: job.urls.clone(),
                    dead: Arc::new(false.into()),
                    assigned_to: proxy_id.to_owned(),
                    job_dispatched: Utc::now().naive_utc(),
                });
                *job = proxy_job;
                info!("re-distributing a dead job for {proxy_id}!");
                return Some(job.to_owned());
            }
        }

        let ip = self.last_ip.fetch_add(IP_AMOUNT, Ordering::Relaxed);
        let mut urls = Vec::with_capacity(IP_AMOUNT as usize);
        for i in 0..IP_AMOUNT {
            let url = Url::parse(&format!("https://{}:443", Ipv4Addr::from(ip + i))).unwrap();
            urls.push(url);
        }
        info!(
            "generating new job with ip-range: {ip}-{} for {proxy_id}",
            ip + IP_AMOUNT
        );
        let proxy_job = Arc::new(ProxyJob {
            urls,
            dead: Arc::new(AtomicBool::new(false)),
            assigned_to: proxy_id.to_owned(),
            job_dispatched: Utc::now().naive_utc(),
        });
        self.active_jobs
            .insert(proxy_id.to_owned(), proxy_job.clone());
        Some(proxy_job)
    }

    async fn complete_job(
        &self,
        proxy_id: &str,
        proxy_outputs: &[ProxyOutput],
        db_pool: Pool<Postgres>,
    ) -> Result<(), Error> {
        info!("job completed: {proxy_id}");
        let job = {
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
                && (job.job_dispatched + Duration::minutes(5)) < now
            {
                job.deref().dead.store(true, Ordering::Relaxed);
            }
        }
    }
}
