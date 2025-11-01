use env_logger::Env;
use log::info;
use std::{future::pending, net::SocketAddr};

use crate::env::ENVVARS;

pub mod env;

pub mod kv_db;

mod public_endpoints;
pub use public_endpoints::public_endpoints;

mod private_endpoints;
pub use private_endpoints::private_endpoints;

mod scrapper_state;
pub use scrapper_state::ScrapperState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ENVVARS.rust_log;
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let public_addr = SocketAddr::new(
        ENVVARS.public_harvester_address,
        ENVVARS.public_harvester_port,
    );
    tokio::spawn(async move {
        let public_listener = tokio::net::TcpListener::bind(public_addr).await.unwrap();
        info!("public server running on {public_addr}!");
        axum::serve(public_listener, public_endpoints())
            .await
            .unwrap();
    });

    let private_addr = SocketAddr::new(
        ENVVARS.private_harvester_address,
        ENVVARS.private_harvester_port,
    );
    tokio::spawn(async move {
        let private_listener = tokio::net::TcpListener::bind(private_addr).await.unwrap();
        info!("private server running on {private_addr}!");
        axum::serve(private_listener, private_endpoints())
            .await
            .unwrap();
    });

    pending::<()>().await;
    Ok(())
}
