use axum::Router;
use env_logger::Env;
use log::info;
use sqlx::{Pool, Postgres};
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

mod error;
pub use error::Error;

mod create_postgres_pool;
pub use create_postgres_pool::create_postgres_pool;

#[derive(Clone)]
pub struct AppState {
    db_pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ENVVARS.rust_log;
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let db_pool = create_postgres_pool(
        &ENVVARS.postgres_user,
        &ENVVARS.postgres_password,
        &ENVVARS.db_address,
        ENVVARS.db_port,
        &ENVVARS.postgres_name,
        ENVVARS.pool_max_conn,
    )
    .await?;

    let app_state = AppState { db_pool };

    let public_addr = SocketAddr::new(
        ENVVARS.public_harvester_address,
        ENVVARS.public_harvester_port,
    );
    let pub_app_state = app_state.clone();
    tokio::spawn(async move {
        let public_listener = tokio::net::TcpListener::bind(public_addr).await.unwrap();
        let router = Router::new()
            .merge(public_endpoints())
            .with_state(pub_app_state);
        info!("public server running on {public_addr}!");
        axum::serve(public_listener, router).await.unwrap();
    });

    let private_addr = SocketAddr::new(
        ENVVARS.private_harvester_address,
        ENVVARS.private_harvester_port,
    );
    let priv_app_state = app_state.clone();
    tokio::spawn(async move {
        let private_listener = tokio::net::TcpListener::bind(private_addr).await.unwrap();
        let router = Router::new()
            .merge(private_endpoints())
            .with_state(priv_app_state);
        info!("private server running on {private_addr}!");
        axum::serve(private_listener, router).await.unwrap();
    });

    pending::<()>().await;
    Ok(())
}
