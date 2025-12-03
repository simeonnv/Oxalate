use crate::global_scan::GlobalScan;
use crate::save_proxy_outputs;
use chrono::Duration;
use chrono::NaiveDateTime;
use log::error;
use log::info;
use oxalate_kv_db::kv_db::KvDb;
use oxalate_urls::Urls;
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
use utoipa::ToSchema;

use oxalate_kv_db::kv_db::Error as KvError;

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
    fn complete_job(
        &self,
        proxy_id: &str,
        proxy_outputs: &[ProxyOutput],
        db_pool: Pool<Postgres>,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn check_for_dead_jobs(&self);
}

#[derive(Serialize, Deserialize)]
pub struct ProxyJob {
    pub urls: Urls,
    pub dead: AtomicBool,
    pub assigned_to: String,
    pub job_dispatched: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
pub enum ProxyOutput {
    HttpBased(HttpBasedOutput),
    Msp(MspOutput),
}

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
pub struct HttpBasedOutput {
    pub url: Url,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
pub struct MspOutput {
    pub url: Url,
    pub online: bool,
    pub online_players_count: i64,
    pub max_online_players: i64,
    pub description: String,
    pub players: Option<Vec<String>>,
    pub version: String,
    pub mods: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ScrapperController {
    pub enabled: AtomicBool,
    pub current_stage: Stage,
}

impl ScrapperController {
    pub fn load(kv_db: &KvDb) -> Result<Arc<Self>, Error> {
        let scrapper_controller = kv_db.get(SCRAPPER_STATE_KEY)?;
        let scrapper_state = match scrapper_controller {
            Some(e) => e,
            None => {
                let scrapper_state = Self::default();
                Self::save_state(&scrapper_state, kv_db)?;
                scrapper_state
            }
        };
        let scrapper_controller = Arc::new(scrapper_state);

        {
            let scrapper_controller = scrapper_controller.clone();
            let kv_db = kv_db.to_owned();
            tokio::spawn(async move {
                loop {
                    sleep(Duration::minutes(5).to_std().unwrap()).await;
                    info!("saving scrapper state!");
                    if let Err(err) = scrapper_controller.save_state(&kv_db) {
                        error!("failed to save scrapper state in background thread!: {err}");
                    }
                }
            });
        }

        Ok(scrapper_controller)
    }

    pub fn save_state(&self, kv_db: &KvDb) -> Result<(), Error> {
        kv_db.insert(SCRAPPER_STATE_KEY, self)?;
        kv_db.flush()?;
        Ok(())
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
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

    pub fn reset(&mut self) {
        *self = Self {
            enabled: false.into(),
            current_stage: match self.current_stage {
                Stage::GlobalScan(_) => Stage::GlobalScan(GlobalScan::default()),
            },
        };
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
    SaveProxyOutput(#[from] save_proxy_outputs::Error),
}
