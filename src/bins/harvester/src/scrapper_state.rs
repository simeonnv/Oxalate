use crate::handle_proxy_outputs;
use crate::kv_db::DB;
use crate::kv_db::Error as KvError;
use crate::save_proxy_outputs;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::Utc;
use dashmap::DashMap;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use thiserror::Error;
use tokio::time::sleep;
use url::Url;

const SCRAPPER_STATE_KEY: &[u8; 14] = b"scrapper state";

#[derive(Serialize, Deserialize, Clone)]
pub enum Stage {
    GlobalScan(GlobalScan),
}

impl Default for Stage {
    fn default() -> Self {
        Self::GlobalScan(GlobalScan::default())
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct GlobalScan {
    pub active_jobs: DashMap<String, Arc<ProxyJob>>,
    pub last_ip: Arc<AtomicU32>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ProxyJob {
    pub urls: Vec<Url>,
    pub dead: Arc<AtomicBool>,
    pub assigned_to: String,
    pub job_dispatched: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProxyOutput {
    pub url: Url,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ScrapperState {
    pub enabled: AtomicBool,
    pub current_stage: Stage,
}

impl ScrapperState {
    pub fn load() -> Result<Arc<Self>, Error> {
        let scrapper_state = DB.get(SCRAPPER_STATE_KEY)?;
        let scrapper_state = match scrapper_state {
            Some(e) => e,
            None => {
                let scrapper_state = Self::default();
                Self::save_state(&scrapper_state)?;
                scrapper_state
            }
        };
        let scrapper_state = Arc::new(scrapper_state);

        {
            let scrapper_state = scrapper_state.clone();
            tokio::spawn(async move {
                loop {
                    sleep(Duration::minutes(5).to_std().unwrap()).await;
                    scrapper_state.check_for_dead_jobs();
                    if let Err(err) = scrapper_state.save_state() {
                        error!("failed to save scrapper state in background thread!: {err}");
                    }
                }
            });
        }

        Ok(scrapper_state)
    }

    pub fn save_state(&self) -> Result<(), Error> {
        DB.insert(SCRAPPER_STATE_KEY, self)?;
        DB.flush()?;
        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&mut self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn req_addresses(&self, proxy_id: &str) -> Option<Arc<ProxyJob>> {
        if !self.enabled.load(Ordering::Relaxed) {
            return None;
        }
        let global_scan = match &self.current_stage {
            Stage::GlobalScan(gs) => gs,
        };
        if let Some(e) = global_scan.active_jobs.get(proxy_id) {
            return Some(e.to_owned());
        }
        for mut job in global_scan.active_jobs.iter_mut() {
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

        let ip = global_scan.last_ip.fetch_add(256, Ordering::Relaxed);
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
        global_scan
            .active_jobs
            .insert(proxy_id.to_owned(), proxy_job.clone());
        Some(proxy_job)
    }

    pub async fn complete_job(
        &self,
        proxy_id: &str,
        proxy_outputs: &[ProxyOutput],
        db_pool: Pool<Postgres>,
    ) -> Result<(), Error> {
        let Stage::GlobalScan(global_scan) = &self.current_stage;
        global_scan
            .active_jobs
            .contains_key(proxy_id)
            .then_some(())
            .ok_or(Error::DeviceHasNoJob)?;

        save_proxy_outputs(proxy_outputs, db_pool).await?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        *self = Self {
            enabled: false.into(),
            current_stage: self.current_stage.clone(),
        };
        self.save_state()?;
        Ok(())
    }

    pub fn check_for_dead_jobs(&self) {
        if let Stage::GlobalScan(global_scan) = &self.current_stage {
            for job in global_scan.active_jobs.iter_mut() {
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
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("kv error -> {0}")]
    Kv(#[from] KvError),

    #[error("The device has no registrated proxy job!")]
    DeviceHasNoJob,

    #[error("failed to save proxy outputs -> {0}")]
    SaveProxyOutput(#[from] handle_proxy_outputs::Error),
}
