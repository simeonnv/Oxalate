use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{ProxyId, save_proxy_outputs};
use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime, Utc};
use dashmap::DashMap;
use enum_dispatch::enum_dispatch;
use exn::*;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use url::Url;
use utoipa::ToSchema;

#[enum_dispatch]
#[async_trait]
pub trait ProxyTaskGenerator<Err: exn::Error> {
    async fn generate_task<LoggingCTX: Serialize + Send + Sync>(
        &self,
        logging_ctx: &LoggingCTX,
    ) -> Result<Option<ProxyTask>, Err>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScraperController {
    pub enabled: AtomicBool,
    pub active_tasks: DashMap<ProxyId, ActiveProxyTask>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ProxyTask(pub Box<[ProxyReq]>);

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub enum ProxyReq {
    Http(HttpReq),
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct HttpReq {
    pub url: Url,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub method: HttpMethod,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub enum ProxyRes {
    HttpRes(HttpRes),
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct HttpRes {
    pub url: Url,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveProxyTask {
    pub created_at: NaiveDateTime,
    pub last_reallocated: NaiveDateTime,
    pub dead: bool,
    pub task: Arc<ProxyTask>,
}

impl ScraperController {
    pub fn new() -> Self {
        Self {
            enabled: false.into(),
            active_tasks: DashMap::new(),
        }
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub async fn get_task<
        PTGError: exn::Error,
        PTG: ProxyTaskGenerator<PTGError>,
        LoggingCTX: Serialize + Send + Sync,
    >(
        &self,
        proxy_id: &ProxyId,
        task_generator: &PTG,
        logging_ctx: &LoggingCTX,
    ) -> Result<Option<Arc<ProxyTask>>, Error> {
        info!(ctx:serde = logging_ctx; "called get_task at scraper controller");
        if !self.enabled.load(Ordering::Relaxed) {
            log::info!(ctx:serde = logging_ctx; "Not creating a task bc scraper controller is disabled");
            return Ok(None);
        }

        if let Some(active_task) = self.active_tasks.get(proxy_id)
            && !active_task.dead
        {
            info!(ctx:serde = logging_ctx; "Returning existing active task");
            return Ok(Some(active_task.value().task.to_owned()));
        }

        let task = task_generator
            .generate_task(logging_ctx)
            .await
            .or_raise(|| Error::TaskGeneratorFailed(std::any::type_name::<PTG>()))?;

        let task = match task {
            Some(e) => e,
            None => {
                info!(ctx:serde = logging_ctx; "generator returned no task");
                return Ok(None);
            }
        };
        let task = Arc::new(task);

        let now = Utc::now().naive_utc();
        let active_task = ActiveProxyTask {
            created_at: now,
            last_reallocated: now,
            dead: false,
            task: task.to_owned(),
        };

        self.active_tasks.insert(proxy_id.to_owned(), active_task);

        info!(ctx:serde = logging_ctx; "successfully created and registered a task");
        Ok(Some(task))
    }

    pub async fn mark_dead_tasks<LoggingCTX: Serialize>(
        &self,
        death_duration: &Duration,
        logging_ctx: &LoggingCTX,
    ) -> Box<[ProxyId]> {
        info!(ctx:serde = logging_ctx; "starting to mark dead tasks");

        let mut dead_tasks_proxy_ids = vec![];
        debug!(ctx:serde = logging_ctx; "getting active tasks mut iter lock");
        for mut task in self.active_tasks.iter_mut() {
            let now = Utc::now().naive_utc();
            if !task.dead && task.created_at + *death_duration > now {
                debug!(ctx:serde = logging_ctx; "getting task mut value lock");
                let mut_active_task = task.value_mut();
                mut_active_task.dead = true;
                dead_tasks_proxy_ids.push(task.key().to_owned());
            }
        }

        info!(ctx:serde = logging_ctx; "marked dead tasks");
        dead_tasks_proxy_ids.into_boxed_slice()
    }

    pub async fn complete_task<LoggingCTX: Serialize>(
        &self,
        proxy_id: &ProxyId,
        proxy_res: &[ProxyRes],
        db_pool: &Pool<Postgres>,
        logging_ctx: &LoggingCTX,
    ) -> Result<(), Error> {
        info!(ctx:serde = logging_ctx; "called complete task at scraper controller");

        if self.active_tasks.get(proxy_id).is_none() {
            info!(ctx:serde = logging_ctx; "A proxy tried to send a task output without having a task assigned");
            return Ok(());
        }

        save_proxy_outputs(proxy_id, proxy_res, db_pool, logging_ctx)
            .await
            .or_raise(|| Error::FailedToSaveProxyRes)?;

        self.active_tasks.remove(proxy_id);

        info!(ctx:serde = logging_ctx; "completed and saved task");
        Ok(())
    }
}

impl Default for ScraperController {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Scraper constroller failed to generate a task from task generator: {0}")]
    TaskGeneratorFailed(&'static str),

    #[error("failed to save proxy responses")]
    FailedToSaveProxyRes,
}

// #[async_trait]
// impl ProxyTaskGenerator<Infallible> for () {
//     async fn generate_task<LoggingCTX: Serialize + Send + Sync>(
//         &self,
//         logging_ctx: &LoggingCTX,
//     ) -> Result<Option<ProxyTask>, Infallible> {
//         error!(ctx:serde = logging_ctx; "USING UNIT PROXY TASK GENERATOR, RETURNING NONE");
//         Ok(None)
//     }
// }
