use std::net::SocketAddr;

use env_logger::Env;

use log::info;

use crate::env::ENVVARS;

pub mod env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ENVVARS.rust_log;
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let addr = SocketAddr::new(ENVVARS.harvester_address, ENVVARS.harvester_port);

    Ok(())
}
