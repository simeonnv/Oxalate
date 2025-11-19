use crate::GlobalScan;
use crate::handle_proxy_outputs;
use crate::kv_db::DB;
use crate::kv_db::Error as KvError;
use chrono::Duration;
use chrono::NaiveDateTime;
use log::error;
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
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

pub trait ScraperLevel {
    fn req_addresses(&self, proxy_id: &str) -> Option<Arc<ProxyJob>>;
    async fn complete_job(
        &self,
        proxy_id: &str,
        proxy_outputs: &[ProxyOutput],
        db_pool: Pool<Postgres>,
    ) -> Result<(), Error>;
    fn check_for_dead_jobs(&self);
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
                    info!("saving scrapper state!");
                    // scrapper_state.check_for_dead_jobs();
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
        match self.current_stage {
            Stage::GlobalScan(ref global_scan) => global_scan.req_addresses(proxy_id),
        }
    }

    pub async fn complete_job(
        &self,
        proxy_id: &str,
        proxy_outputs: &[ProxyOutput],
        db_pool: Pool<Postgres>,
    ) -> Result<(), Error> {
        match self.current_stage {
            Stage::GlobalScan(ref global_scan) => {
                global_scan
                    .complete_job(proxy_id, proxy_outputs, db_pool)
                    .await?
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        *self = Self {
            enabled: false.into(),
            current_stage: match self.current_stage {
                Stage::GlobalScan(_) => Stage::GlobalScan(GlobalScan::default()),
            },
        };
        self.save_state()?;
        Ok(())
    }

    pub fn check_for_dead_jobs(&self) {
        match self.current_stage {
            Stage::GlobalScan(ref global_scan) => global_scan.check_for_dead_jobs(),
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
