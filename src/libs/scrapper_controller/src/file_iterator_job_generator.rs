use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::Utc;
use oxalate_urls::urls::{HttpBasedContent, HttpMethod, ProxyReq, ProxyReqs};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    Error, ProxyId,
    scrapper_controller::{ProxyJob, ScraperJobGenerator},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileIteratorJobGenerator {
    urls: Box<[Url]>,
    index: Mutex<u32>,
}

impl FileIteratorJobGenerator {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let file = fs::read_to_string(path).map_err(|e| Error::FileReadError(e.to_string()))?;
        let urls = file
            .lines()
            .into_iter()
            .filter_map(|e| Url::parse(e).ok())
            .collect::<Box<_>>();

        Ok(Self {
            urls,
            index: 0.into(),
        })
    }
}

impl ScraperJobGenerator for FileIteratorJobGenerator {
    fn generate_new_job(&self, proxy_id: &ProxyId, job_size: u32) -> Option<Arc<ProxyJob>> {
        let mut lock = self.index.lock().ok()?;

        let start = *lock;
        let mut end = *lock + job_size;

        if end >= self.urls.len() as u32 {
            end = self.urls.len() as u32;
            if end - start == 0 {
                drop(lock);
                return None;
            }
        }
        *lock = end;
        drop(lock);

        let mut reqs = vec![];
        for url in &self.urls[start as usize..end as usize] {
            let req = ProxyReq::Https(HttpBasedContent {
                url: url.to_owned(),
                headers: HashMap::new(),
                body: None,
                method: HttpMethod::Get,
            });
            reqs.push(req);
        }
        let reqs = ProxyReqs(reqs);

        Some(Arc::new(ProxyJob {
            reqs: reqs,
            dead: false.into(),
            assigned_to: proxy_id.to_owned(),
            job_dispatched: Utc::now().naive_local(),
        }))
    }
}
