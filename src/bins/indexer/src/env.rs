use std::{net::IpAddr, path::Path};

use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct EnvVars {
    #[envconfig(from = "RUST_LOG")]
    pub rust_log: String,

    #[envconfig(from = "INDEXER_ADDRESS", default = "0.0.0.0")]
    pub indexer_address: IpAddr,

    #[envconfig(from = "INDEXER_PORT", default = "22267")]
    pub indexer_port: u16,

    #[envconfig(from = "POSTGRES_NAME")]
    pub postgres_name: String,

    #[envconfig(from = "POSTGRES_USER")]
    pub postgres_user: String,

    #[envconfig(from = "POSTGRES_PASSWORD")]
    pub postgres_password: String,

    #[envconfig(from = "POOL_MAX_CONN", default = "5")]
    pub pool_max_conn: u32,

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

    #[envconfig(from = "KAFKA_OUTLET_LOGS_TOPIC", default = "harvester_logs")]
    pub kafka_harvester_logs_topic: String,
}

pub fn load_env_vars() -> EnvVars {
    let env_path = if cfg!(debug_assertions) {
        Path::new("./.env.dev")
    } else {
        Path::new("./.env")
    };

    if let Err(e) = dotenv::from_path(env_path) {
        eprintln!("Failed to load {} file: {}", env_path.display(), e);
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
