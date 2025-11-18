use std::{
    net::Ipv4Addr,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use chrono::{Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use url::Url;

use crate::{
    save_proxy_outputs,
    scrapper_state::{Error, ProxyJob, ProxyOutput, ScraperLevel},
};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct GlobalScan {
    pub active_jobs: DashMap<String, Arc<ProxyJob>>,
    pub last_ip: Arc<AtomicU32>,
}

impl ScraperLevel for GlobalScan {
    fn req_addresses(&self, proxy_id: &str) -> Option<Arc<ProxyJob>> {
        if let Some(e) = self.active_jobs.get(proxy_id) {
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

                return Some(job.to_owned());
            }
        }

        let ip = self.last_ip.fetch_add(256, Ordering::Relaxed);
        let mut urls = Vec::with_capacity(256);
        for i in 0..256 {
            let url = Url::parse(&format!("https://{}", Ipv4Addr::from(ip + i))).unwrap();
            urls.push(url);
        }
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
        self.active_jobs
            .contains_key(proxy_id)
            .then_some(())
            .ok_or(Error::DeviceHasNoJob)?;
        save_proxy_outputs(proxy_outputs, db_pool).await?;
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
