use dashmap::DashMap;
use oxalate_scraper_controller::ProxyId;
use tokio::sync::watch::{self, Receiver, Sender};

#[derive(Clone, Debug)]
pub struct ProxyConnectionStore {
    pub inner: DashMap<ProxyId, (Sender<bool>, Receiver<bool>)>,
}

impl ProxyConnectionStore {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    pub async fn subscribe(&self, proxy_id: ProxyId) -> watch::Receiver<bool> {
        self.inner
            .entry(proxy_id)
            .or_insert(watch::channel(false))
            .to_owned()
            .1
    }

    pub async fn connected(&self, proxy_id: ProxyId) {
        let _ = self
            .inner
            .entry(proxy_id)
            .or_insert(watch::channel(true))
            .0
            .send(true);
    }
    pub async fn disconnected(&self, proxy_id: ProxyId) {
        let _ = self
            .inner
            .entry(proxy_id)
            .or_insert(watch::channel(false))
            .0
            .send(false);
    }
}

impl Default for ProxyConnectionStore {
    fn default() -> Self {
        Self::new()
    }
}
