use std::{
    collections::{HashMap, VecDeque},
    convert::Infallible,
    fs,
    path::PathBuf,
    usize,
};

use async_trait::async_trait;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::scraper_controller::{HttpMethod, HttpReq, ProxyReq, ProxyTask, ProxyTaskGenerator};
use thiserror::Error;

use exn::{Result, ResultExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileIteratorTaskGenerator {
    urls: VecDeque<Box<[Url]>>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to build file iterator task generator")]
    FailedToBuild,
}

impl FileIteratorTaskGenerator {
    pub fn new(path: &PathBuf, job_size: usize) -> Result<Self, Error> {
        let file = fs::read_to_string(path).or_raise(|| Error::FailedToBuild)?;
        let urls = file
            .lines()
            .filter_map(|e| Url::parse(e).ok())
            .collect::<Box<_>>();

        let mut queue = VecDeque::with_capacity(urls.len() / job_size);
        for chunk in urls.chunks(job_size) {
            let chunk = chunk.to_vec().into_boxed_slice();
            queue.push_back(chunk);
        }
        let queue = queue;

        Ok(Self { urls: queue })
    }
}

#[async_trait]
impl ProxyTaskGenerator<Infallible> for FileIteratorTaskGenerator {
    async fn generate_task<LoggingCTX: Serialize + Send + Sync>(
        &mut self,
        logging_ctx: &LoggingCTX,
    ) -> Result<Option<ProxyTask>, Infallible> {
        let urls = self.urls.pop_front();
        let urls = match urls {
            Some(e) => e,
            None => {
                debug!(
                    ctx:serde = logging_ctx;
                    "File task generator is done; No more urls in file"
                );
                return Ok(None);
            }
        };

        let reqs = urls
            .into_iter()
            .map(|e| {
                let http_req = HttpReq {
                    url: e,
                    body: String::new(),
                    headers: HashMap::new(),
                    method: HttpMethod::Get,
                };

                ProxyReq::HttpReq(http_req)
            })
            .collect();

        let task = ProxyTask(reqs);

        info!(
            ctx:serde = logging_ctx;
            "Successfully popped a new proxy task from file iterator task generator!"
        );
        Ok(Some(task))
    }
}
