use axum::Router;
use chrono::NaiveDateTime;
use dashmap::{DashMap, DashSet};
use env_logger::Env;
use log::info;
use sqlx::{Pool, Postgres};
use std::{
    future::pending,
    net::SocketAddr,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::{select, signal};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

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

mod handle_proxy_outputs;
pub use handle_proxy_outputs::save_proxy_outputs;

mod insure_device_exists;
pub use insure_device_exists::insure_device_exists;

mod global_scan;
pub use global_scan::GlobalScan;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub scrapper_state: Arc<ScrapperState>,
    pub uptime_connected_devices: Arc<DashMap<String, NaiveDateTime>>,
    pub shutdown: Arc<Shutdown>,
}

#[derive(Default)]
pub struct Shutdown {
    pub task_tracker: TaskTracker,
    pub token: CancellationToken,
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

    let scrapper_state = ScrapperState::load()?;
    let app_state = AppState {
        db_pool,
        scrapper_state,
        uptime_connected_devices: Arc::new(DashMap::new()),
        shutdown: Arc::new(Shutdown::default()),
    };

    let public_addr = SocketAddr::new(
        ENVVARS.public_harvester_address,
        ENVVARS.public_harvester_port,
    );
    let pub_app_state = app_state.clone();
    let pub_shutdown = app_state.shutdown.to_owned();
    tokio::spawn(async move {
        let public_listener = tokio::net::TcpListener::bind(public_addr).await.unwrap();
        let router = Router::new()
            .merge(public_endpoints())
            .with_state(pub_app_state);

        info!("public server running on {public_addr}!");
        axum::serve(public_listener, router)
            .with_graceful_shutdown(shutdown_signal(pub_shutdown))
            .await
            .unwrap();
    });

    let private_addr = SocketAddr::new(
        ENVVARS.private_harvester_address,
        ENVVARS.private_harvester_port,
    );
    let priv_app_state = app_state.clone();
    let priv_shutdown = app_state.shutdown.to_owned();
    tokio::spawn(async move {
        let private_listener = tokio::net::TcpListener::bind(private_addr).await.unwrap();
        let router = Router::new()
            .merge(private_endpoints())
            .with_state(priv_app_state);
        info!("private server running on {private_addr}!");
        axum::serve(private_listener, router)
            .with_graceful_shutdown(shutdown_signal(priv_shutdown))
            .await
            .unwrap();
    });

    app_state.shutdown.task_tracker.wait().await;
    info!("shutting down!");

    Ok(())
}

async fn shutdown_signal(shutdown: Arc<Shutdown>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received!");
    shutdown.token.cancel();
    tokio::time::sleep(Duration::from_secs(1)).await;
    shutdown.task_tracker.close();
}
