use crate::kv_db::DB;
use crate::kv_db::Error as KvError;
use axum::http::HeaderMap;
use axum::http::Response;
use axum::http::StatusCode;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::Utc;
use dashmap::DashMap;
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
use tokio::spawn;
use url::Url;
use uuid::Uuid;

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
    urls: Vec<Url>,
    dead: Arc<AtomicBool>,
    assigned_to: String,
    job_dispatched: NaiveDateTime,
}

pub struct ProxyOutput {
    pub url: Url,
    pub status: StatusCode,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ScrapperState {
    pub enabled: bool,
    pub current_stage: Stage,
}

impl ScrapperState {
    pub fn load() -> Result<Self, Error> {
        let scrapper_state = DB.get(SCRAPPER_STATE_KEY)?;
        let scrapper_state = match scrapper_state {
            Some(e) => e,
            None => {
                let scrapper_state = Self::default();
                Self::save_state(&scrapper_state)?;
                scrapper_state
            }
        };
        Ok(scrapper_state)
    }

    pub fn save_state(&self) -> Result<(), Error> {
        DB.insert(SCRAPPER_STATE_KEY, self)?;
        DB.flush()?;
        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled = true
    }

    pub fn req_addresses(&mut self, proxy_id: &str) -> Option<Arc<ProxyJob>> {
        if !self.enabled {
            return None;
        }
        let global_scan = match &mut self.current_stage {
            Stage::GlobalScan(gs) => gs,
        };
        if let Some(e) = global_scan.active_jobs.get(proxy_id) {
            return Some(e.to_owned());
        }
        for mut job in global_scan.active_jobs.iter_mut() {
            let (id, job) = job.pair_mut();
            if job.dead.load(Ordering::Relaxed) {
                let proxy_job = Arc::new(ProxyJob {
                    urls: job.urls.clone(),
                    dead: Arc::new(AtomicBool::new(false)),
                    assigned_to: id.to_owned(),
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
        completed_job: Box<[ProxyOutput]>,
        db_pool: Pool<Postgres>,
    ) -> Result<(), Error> {
        let Stage::GlobalScan(global_scan) = &self.current_stage;
        let job = global_scan
            .active_jobs
            .get(proxy_id)
            .ok_or(Error::NoSuchProxyJob)?;

        let mut join_handles = vec![];
        for output in completed_job {
            let db_pool = db_pool.clone();
            let handle = spawn(async move {
                let uuid = Uuid::new_v4();
                let body = output.body;
                let headers = serde_json::to_value(&output.headers).unwrap();

                sqlx::query!(
                    "
                      INSERT INTO Webpages
                            (webpage_id, body, headers)
                      VALUES ($1, $2, $3)
                   ",
                    uuid,
                    body,
                    headers
                )
                .execute(&db_pool)
                .await
            });
            join_handles.push(handle);
        }

        for handle in join_handles {
            let _ = handle.await;
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        *self = Self {
            enabled: false,
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
    KvError(#[from] KvError),

    #[error("No such registrated proxy job!")]
    NoSuchProxyJob,
}
