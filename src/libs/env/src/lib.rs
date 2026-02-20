use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct EnvVars {
    #[envconfig(from = "RUST_LOG", default = "info")]
    pub rust_log: String,

    // Public Harvester
    #[envconfig(from = "PUBLIC_HARVESTER_BIND_ADDRESS", default = "0.0.0.0")]
    pub public_harvester_bind_address: IpAddr,
    #[envconfig(from = "PUBLIC_HARVESTER_PORT", default = "6767")]
    pub public_harvester_port: u16,
    // Private Harvester
    #[envconfig(from = "PRIVATE_HARVESTER_BIND_ADDRESS", default = "0.0.0.0")]
    pub private_harvester_bind_address: IpAddr,
    #[envconfig(from = "PRIVATE_HARVESTER_PORT", default = "6969")]
    pub private_harvester_port: u16,
    // Harvester
    #[envconfig(from = "HARVESTER_DNS", default = "oxalate_harvester")]
    pub public_harvester_dns: String,

    // Indexer
    #[envconfig(from = "INDEXER_BIND_ADDRESS", default = "0.0.0.0")]
    pub indexer_bind_address: IpAddr,
    #[envconfig(from = "INDEXER_PORT", default = "22267")]
    pub indexer_port: u16,
    #[envconfig(from = "INDEXER_DNS", default = "oxalate_indexer")]
    pub indexer_dns: String,

    // Database
    #[envconfig(from = "POSTGRES_USER")]
    pub postgres_user: String,
    #[envconfig(from = "POSTGRES_PASSWORD")]
    pub postgres_password: String,
    #[envconfig(from = "POSTGRES_NAME")]
    pub postgres_name: String,
    #[envconfig(from = "DB_BIND_ADDRESS", default = "0.0.0.0")]
    pub db_bind_address: IpAddr,
    #[envconfig(from = "DB_DNS", default = "oxalate-paradedb")]
    pub db_dns: String,
    #[envconfig(from = "DB_PORT", default = "6666")]
    pub db_port: u16,
    #[envconfig(from = "POOL_MAX_CONN", default = "25")]
    pub pool_max_conn: u32,

    // Kafka
    #[envconfig(from = "KAFKA_BIND_ADDRESS", default = "0.0.0.0")]
    pub kafka_bind_address: IpAddr,
    #[envconfig(from = "KAFKA_PORT", default = "19092")]
    pub kafka_port: u16,
    #[envconfig(from = "KAFKA_DNS")] // , default = "oxalate_redpanda"
    pub kafka_dns: Option<String>, // depending if this is none you can disable kafka logging
    #[envconfig(from = "KAFKA_MESSAGE_TIMEOUT_MS", default = "5000")]
    pub kafka_message_timeout_ms: u64,

    #[envconfig(from = "KAFKA_HARVESTER_LOGS_TOPIC", default = "harvester_logs")]
    pub kafka_harvester_logs_topic: String,
    #[envconfig(from = "KAFKA_INDEXER_LOGS_TOPIC", default = "indexer_logs")]
    pub kafka_indexer_logs_topic: String,
    #[envconfig(from = "KAFKA_OUTLET_LOGS_TOPIC", default = "outlet_logs")]
    pub kafka_outlet_logs_topic: String,

    // Filesystem
    #[envconfig(from = "URLS_FILE", default = "./urls.txt")]
    pub urls_file: PathBuf,
}

pub fn load_env_vars() -> EnvVars {
    if cfg!(debug_assertions) {
        let dev_env_path = Path::new("./.env.dev");
        if let Err(e) = dotenv::from_path(dev_env_path) {
            println!(
                "Failed to load env file: {} file: {}",
                dev_env_path.display(),
                e
            );
        }
    }

    let env_vars = EnvVars::init_from_env();
    match env_vars {
        Ok(e) => e,
        Err(e) => panic!("failed to load env vars: {}", e),
    }
}

lazy_static::lazy_static! {
    pub static ref ENVVARS: EnvVars = load_env_vars();
}
