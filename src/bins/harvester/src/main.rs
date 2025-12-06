use axum::Router;
use env_logger::Env;
use log::info;
use oxalate_kv_db::kv_db::KvDb;
use oxalate_scrapper_controller::ScrapperController;
use sqlx::{Pool, Postgres};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tower_http::trace::TraceLayer;

use crate::env::ENVVARS;

pub mod env;

pub mod public_endpoints;
pub use public_endpoints::public_endpoints;

pub mod private_endpoints;
pub use private_endpoints::private_endpoints;

mod error;
pub use error::Error;

mod create_postgres_pool;
pub use create_postgres_pool::create_postgres_pool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub scrapper_state: Arc<ScrapperController>,
    pub shutdown: Arc<Shutdown>,
    pub kv_db: KvDb,
}

#[derive(Default)]
pub struct Shutdown {
    pub task_tracker: TaskTracker,
    pub token: CancellationToken,
}

#[tokio::main(flavor = "multi_thread")]
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

    let kv_db = KvDb::new(&PathBuf::from("./db")).unwrap();
    let app_state_kv_db = kv_db.clone();
    let scrapper_state = ScrapperController::load(&kv_db)?;
    scrapper_state.enable();

    let app_state = AppState {
        db_pool,
        scrapper_state,
        shutdown: Arc::new(Shutdown::default()),
        kv_db: app_state_kv_db,
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
            .with_state(pub_app_state)
            .layer(TraceLayer::new_for_http());

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
            .with_state(priv_app_state)
            .layer(TraceLayer::new_for_http());
        info!("private server running on {private_addr}!");
        axum::serve(private_listener, router)
            .with_graceful_shutdown(shutdown_signal(priv_shutdown))
            .await
            .unwrap();
    });

    app_state.shutdown.task_tracker.wait().await;
    info!("shutting down!");
    let _ = app_state.scrapper_state.save_state(&kv_db);

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
