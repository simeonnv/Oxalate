use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;
use enum_dispatch::enum_dispatch;
use exn::Result;
use exn::ResultExt;
use oxalate_scraper_controller::{
    FileIteratorTaskGenerator, ProxyId, scraper_controller::ProxyTaskGenerator,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxySettingsStore {
    file_proxy_task_generator: Arc<Mutex<FileIteratorTaskGenerator>>,
    settings: DashMap<ProxyId, ProxySettings>,
}

#[derive(thiserror::Error, Debug)]
pub enum NewError {
    #[error("failed to build proxy settings store bc failed to build task generator iterator")]
    BuildTaskGenerator,
}

impl ProxySettingsStore {
    pub fn new(path: &PathBuf) -> Result<Self, NewError> {
        let file_task_gen =
            FileIteratorTaskGenerator::new(path, 512).or_raise(|| NewError::BuildTaskGenerator)?;

        Ok(Self {
            file_proxy_task_generator: Arc::new(Mutex::new(file_task_gen)),
            settings: DashMap::new(),
        })
    }

    pub fn get_or_create_settings(&self, proxy_id: ProxyId) -> ProxySettings {
        let settings = self
            .settings
            .entry(proxy_id)
            .or_insert_with(|| ProxySettings {
                task_generator: TaskGenerators::FileIteratorTaskGenerator(
                    self.file_proxy_task_generator.to_owned(),
                ),
            });

        settings.to_owned()
    }

    pub fn mutate_settings<F>(&self, proxy_id: ProxyId, f: F)
    where
        F: FnOnce(&mut ProxySettings),
    {
        let mut refmut = self
            .settings
            .entry(proxy_id)
            .or_insert_with(|| ProxySettings {
                task_generator: TaskGenerators::FileIteratorTaskGenerator(
                    self.file_proxy_task_generator.to_owned(),
                ),
            });
        let settings = refmut.value_mut();
        f(settings)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProxySettings {
    task_generator: TaskGenerators,
}

#[enum_dispatch(ProxyTaskGenerator)]
#[derive(Serialize, Deserialize, Debug, Clone)]
enum TaskGenerators {
    FileIteratorTaskGenerator(Arc<Mutex<FileIteratorTaskGenerator>>),
}
