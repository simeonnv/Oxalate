use dashmap::DashMap;
use oxalate_scraper_controller::ProxyId;
use tokio::sync::watch::{self, Receiver, Sender};

#[derive(Clone)]
pub struct ProxyConnectionStore {
    pub store: DashMap<ProxyId, (Sender<bool>, Receiver<bool>)>,
}

impl ProxyConnectionStore {
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }

    pub async fn subscribe(&self, proxy_id: ProxyId) -> watch::Receiver<bool> {
        self.store
            .entry(proxy_id)
            .or_insert(watch::channel(false))
            .to_owned()
            .1
    }

    pub async fn connected(&self, proxy_id: ProxyId) {
        let _ = self
            .store
            .entry(proxy_id)
            .or_insert(watch::channel(false))
            .0
            .send(true);
    }
    pub async fn disconnected(&self, proxy_id: ProxyId) {
        let _ = self
            .store
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
