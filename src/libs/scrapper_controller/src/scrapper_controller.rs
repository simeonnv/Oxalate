use crate::Error;
use crate::ProxyId;
use crate::global_scan::Ipv4IteratorJobGenerator;
use crate::save_proxy_outputs;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::Utc;
use dashmap::DashMap;
use log::error;
use log::info;
use oxalate_kv_db::kv_db::KvDb;
use oxalate_urls::urls::ProxyReqs;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::time::sleep;
use url::Url;
use utoipa::ToSchema;

use oxalate_kv_db::kv_db::Error as KvError;

const SCRAPPER_STATE_KEY: &[u8; 14] = b"scrapper state";

pub trait ScraperJobGenerator {
    fn generate_new_job(&self, proxy_id: &ProxyId) -> Arc<ProxyJob>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyJob {
    pub reqs: ProxyReqs,
    pub dead: AtomicBool,
    pub assigned_to: ProxyId,
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
    pub ping: f64,
    pub mods: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrapperController {
    pub enabled: AtomicBool,
    pub proxies: DashMap<ProxyId, (JobGenerators, Option<Arc<ProxyJob>>)>,
    // generates jobs by iterating over every single possible ipv4 addr
    pub global_ip_job_generator: Ipv4IteratorJobGenerator,
}

impl Default for ScrapperController {
    fn default() -> Self {
        Self {
            enabled: true.into(),
            proxies: Default::default(),
            global_ip_job_generator: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub enum JobGenerators {
    #[default]
    Ipv4Iterator,
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

    pub fn get_job(&self, proxy_id: &ProxyId) -> Option<Arc<ProxyJob>> {
        if !self.enabled.load(Ordering::Relaxed) {
            info!("didnt send job bc server is disabled");
            return None;
        }
        let proxy = self
            .proxies
            .get(proxy_id)
            .map(|e| e.value().to_owned())
            .unwrap_or_else(|| {
                self.register_new_proxy(proxy_id.to_owned());
                (JobGenerators::default(), None)
            });

        if let Some(proxy_job) = &proxy.1 {
            return Some(proxy_job.to_owned());
        }

        if let Some(proxy_job) = self.find_and_reassign_dead_job(proxy_id) {
            return Some(proxy_job);
        }

        let job = match proxy.0 {
            JobGenerators::Ipv4Iterator => self.global_ip_job_generator.generate_new_job(proxy_id),
        };

        let mut proxy_ref = self.proxies.get_mut(proxy_id).unwrap();
        let proxy_val = proxy_ref.value_mut();
        proxy_val.1 = Some(job.to_owned());

        Some(job)
    }

    pub fn register_new_proxy(&self, proxy_id: ProxyId) {
        self.proxies
            .insert(proxy_id, (JobGenerators::default(), None));
    }

    pub async fn complete_job(
        &self,
        proxy_id: &ProxyId,
        proxy_outputs: &[ProxyOutput],
        db_pool: Pool<Postgres>,
    ) -> Result<(), Error> {
        let mut entry = self
            .proxies
            .get_mut(proxy_id)
            .ok_or(Error::ProxyHasNotBeenSeenBefore)?;
        let (_, job) = entry.value_mut();
        if job.is_none() {
            return Err(Error::ProxyHasNoJob);
        }
        *job = None;
        save_proxy_outputs(proxy_id, proxy_outputs, db_pool).await?;
        Ok(())
    }

    pub fn find_and_reassign_dead_job(&self, proxy_id: &ProxyId) -> Option<Arc<ProxyJob>> {
        for mut v in self.proxies.iter_mut() {
            let (proxy_generator, job) = v.value_mut();
            let job = match job.to_owned() {
                Some(e) => e,
                None => continue,
            };
            let proxy_generator = proxy_generator.to_owned();
            drop(v);

            let now = Utc::now().naive_local();

            // i could replace this with a seperate funtion that runs every minute instad of having this run on every job assign
            if job.job_dispatched + Duration::minutes(5) > now {
                job.dead.store(true, Ordering::Relaxed);
            }

            if job.dead.load(Ordering::Relaxed) {
                let new_job = Arc::new(ProxyJob {
                    reqs: job.reqs.to_owned(),
                    dead: false.into(),
                    assigned_to: proxy_id.to_owned(),
                    job_dispatched: now,
                });
                let old_key = job.assigned_to.to_owned();
                drop(job);
                self.proxies.remove(&old_key);
                self.proxies.insert(
                    proxy_id.to_owned(),
                    (proxy_generator, Some(new_job.to_owned())),
                );

                return Some(new_job);
            }
        }
        None
    }
}
