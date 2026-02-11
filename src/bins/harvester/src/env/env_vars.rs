use std::{net::IpAddr, path::PathBuf};

use envconfig::Envconfig;
use lazy_static::lazy_static;

use crate::env::load_env_vars;

#[derive(Envconfig)]
pub struct EnvVars {
    #[envconfig(from = "RUST_LOG")]
    pub rust_log: String,

    #[envconfig(from = "PUBLIC_HARVESTER_ADDRESS", default = "0.0.0.0")]
    pub public_harvester_address: IpAddr,

    #[envconfig(from = "PUBLIC_HARVESTER_PORT", default = "6767")]
    pub public_harvester_port: u16,

    #[envconfig(from = "PRIVATE_HARVESTER_ADDRESS", default = "127.0.0.1")]
    pub private_harvester_address: IpAddr,

    #[envconfig(from = "PRIVATE_HARVESTER_PORT", default = "6969")]
    pub private_harvester_port: u16,

    #[envconfig(from = "HARVESTER_KV_DB_PATH", default = "./db")]
    pub harvester_kv_db_path: PathBuf,

    #[envconfig(from = "POSTGRES_NAME")]
    pub postgres_name: String,

    #[envconfig(from = "POSTGRES_USER")]
    pub postgres_user: String,

    #[envconfig(from = "POSTGRES_PASSWORD")]
    pub postgres_password: String,

    #[envconfig(from = "POOL_MAX_CONN", default = "5")]
    pub pool_max_conn: u32,

    #[envconfig(from = "URLS_PATH")]
    pub urls_path: PathBuf,

    #[envconfig(from = "DB_ADDRESS")]
    pub db_address: String,

    #[envconfig(from = "DB_PORT", default = "5432")]
    pub db_port: u16,

    #[envconfig(from = "KAFKA_ADDRESS")]
    pub kafka_address: Option<IpAddr>,

    #[envconfig(from = "KAFKA_PORT", default = "9092")]
    pub kafka_port: u16,

    #[envconfig(from = "KAFKA_MESSAGE_TIMEOUT_MS", default = "5000")]
    pub kafka_message_timeout_ms: u32,

    #[envconfig(from = "KAFKA_HARVESTER_LOGS_TOPIC", default = "harvester_logs")]
    pub kafka_harvester_logs_topic: String,

    #[envconfig(from = "URLS_FILE")]
    pub urls_file: PathBuf,
}

lazy_static! {
    pub static ref ENVVARS: EnvVars = load_env_vars();
}
