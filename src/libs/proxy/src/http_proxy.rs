use deadqueue::limited::Queue;
pub use reqwest::Method;
use reqwest::{Client, Response};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{task::JoinHandle, time::sleep};

#[derive(Clone)]
pub struct ProxyHttpRequest {
    pub req_type: Method,
    pub url: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}

pub struct ProxyHttpResponse {
    pub url: String,
    pub req_type: Method,
    pub response: Option<Response>,
}

pub struct HttpProxy {
    req_queue: Arc<Queue<ProxyHttpRequest>>,
    res_queue: Arc<Queue<ProxyHttpResponse>>,
    pub request_timeout: Duration,
    task_handle: JoinHandle<()>,
}

impl Drop for HttpProxy {
    fn drop(&mut self) {
        self.task_handle.abort();
    }
}

impl HttpProxy {
    pub fn workload(&self) -> (usize, usize) {
        (self.req_queue.len(), self.req_queue.capacity())
    }

    pub fn new(request_timeout: Duration) -> Self {
        let req_queue = Arc::new(Queue::new(256));
        let res_queue = Arc::new(Queue::new(256));
        let http_client = Client::new();

        let task_handle = tokio::spawn(proxy_task(
            req_queue.clone(),
            res_queue.clone(),
            request_timeout,
            http_client,
        ));

        Self {
            req_queue,
            res_queue,
            request_timeout,
            task_handle,
        }
    }

    pub async fn enqueue(&self, request: &ProxyHttpRequest) {
        self.req_queue.push(request.clone()).await;
    }

    pub async fn bundled_enqueue(&self, requests: &[ProxyHttpRequest]) {
        for req in requests {
            self.enqueue(req).await;
        }
    }

    pub async fn get_res(&self) -> ProxyHttpResponse {
        self.res_queue.pop().await
    }

    pub async fn get_bundled_res(&self) -> Box<[ProxyHttpResponse]> {
        self.res_queue.wait_full().await;
        let mut buff = Vec::with_capacity(256);
        for _ in 0..self.res_queue.len() {
            buff.push(self.res_queue.pop().await);
        }
        buff.into_boxed_slice()
    }
}

async fn proxy_task(
    req_queue: Arc<Queue<ProxyHttpRequest>>,
    res_queue: Arc<Queue<ProxyHttpResponse>>,
    request_timeout: Duration,
    http_client: Client,
) {
    loop {
        let request = req_queue.pop().await;

        let mut req = http_client.request(request.req_type.clone(), request.url.clone());
        for (key, value) in &request.headers {
            req = req.header(key, value);
        }
        req = req.body(request.body.clone());

        let res = req.send().await.ok();

        let proxy_res = ProxyHttpResponse {
            url: request.url,
            req_type: request.req_type,
            response: res,
        };

        res_queue.push(proxy_res).await;
        sleep(request_timeout).await;
    }
}
