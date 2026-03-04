use axum::{Router, middleware::from_fn_with_state};
use envconfig::Envconfig;
use kafka_writer_rs::KafkaLogWriter;
use log::info;
use log_json_serializer::parse_log;
use neo4rs::Graph;
use oxalate_env::load_env_vars;
// use oxalate_env::ENVVARS;
use oxalate_kv_db::kv_db::KvDb;
use oxalate_scraper_controller::ScraperController;
use rdkafka::{ClientConfig, producer::FutureProducer};
use sqlx::{Pool, Postgres};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tower_http::{
    cors::{self, Any},
    trace::TraceLayer,
};

mod proxy_connection_store;
pub use proxy_connection_store::ProxyConnectionStore;

pub mod middleware;
use middleware::logging_middleware::logging_middleware;

pub mod public_endpoints;
pub use public_endpoints::public_endpoints;

pub mod private_endpoints;
pub use private_endpoints::private_endpoints;

mod create_postgres_pool;
pub use create_postgres_pool::create_postgres_pool;

pub mod save_scraper_controller;
pub use save_scraper_controller::save_scraper_controller;

pub mod load_scraper_controller;
pub use load_scraper_controller::load_scraper_controller;

pub const SCRAPER_CONTROLLER_KV_KEY: &str = "scraper controller";

pub mod proxy_settings_store;
use proxy_settings_store::ProxySettingsStore;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub neo4j_pool: Graph,
    pub scraper_controller: Arc<ScraperController>,
    pub proxy_settings_store: Arc<ProxySettingsStore>,
    pub proxy_connection_store: Arc<ProxyConnectionStore>,
    pub shutdown: Arc<Shutdown>,
    pub kafka_outlet_producer: Option<FutureProducer>,
    pub kv_db: KvDb,
}

#[derive(Default)]
pub struct Shutdown {
    pub task_tracker: TaskTracker,
    pub token: CancellationToken,
}

#[derive(Envconfig)]
pub struct EnvVars {
    #[envconfig(from = "RUST_LOG", default = "info")]
    pub rust_log: String,

    // kafka
    #[envconfig(from = "KAFKA_PORT", default = "19092")]
    pub kafka_port: u16,

    #[envconfig(from = "KAFKA_DNS")] // , default = "oxalate_redpanda"
    pub kafka_dns: Option<String>, // depending if this is none you can disable kafka logging

    #[envconfig(from = "KAFKA_MESSAGE_TIMEOUT_MS", default = "5000")]
    pub kafka_message_timeout_ms: u64,

    #[envconfig(from = "KAFKA_HARVESTER_LOGS_TOPIC", default = "harvester_logs")]
    pub kafka_harvester_logs_topic: String,

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

    // neo4j
    #[envconfig(from = "NEO4J_AUTH", default = "neo4j/rootrootroot")]
    pub neo4j_auth: String,
    #[envconfig(from = "NEO4J_PORT", default = "7687")]
    pub neo4j_port: u16,
    #[envconfig(from = "NEO4J_DNS")]
    pub neo4j_dns: String,

    // harvester
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

    #[envconfig(from = "URLS_FILE", default = "./urls.txt")]
    pub urls_file: PathBuf,
}

lazy_static::lazy_static! {
    pub static ref ENVVARS: EnvVars = load_env_vars();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ENVVARS.rust_log;

    let producer: Option<FutureProducer> = ENVVARS.kafka_dns.as_ref().map(|dns| {
        let kafka_connect_url = format!("{}:{}", dns, ENVVARS.kafka_port);
        println!("kafka connect url is: {kafka_connect_url}");
        let client = ClientConfig::new()
            .set("bootstrap.servers", kafka_connect_url)
            .set(
                "message.timeout.ms",
                ENVVARS.kafka_message_timeout_ms.to_string(),
            )
            .create()
            .expect("failed to create kafka client");
        println!("connected to kafka and inited kafka future producer");
        client
    });

    let fern = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}",
                parse_log(message, record).expect("failed to serialize log into json")
            ));
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());
    let fern = {
        match producer {
            Some(ref client) => {
                let kafka_writer = Box::new(
                    KafkaLogWriter::new(client.to_owned(), &ENVVARS.kafka_harvester_logs_topic)
                        .await,
                );

                fern.chain(fern::Output::writer(kafka_writer, "\n"))
            }
            _ => fern,
        }
    };
    fern.apply().expect("failed to create logger");
    log::info!("inited logger");

    let db_pool = create_postgres_pool(
        &ENVVARS.postgres_user,
        &ENVVARS.postgres_password,
        &ENVVARS.db_dns,
        ENVVARS.db_port,
        &ENVVARS.postgres_db,
        ENVVARS.pool_max_conn,
    )
    .await
    .unwrap();

    let neo4j_pool = {
        let mut parts = ENVVARS.neo4j_auth.split("/");
        let user = parts.next().unwrap_or("root");
        let password = parts.next().unwrap_or("root");
        let url = format!("{}:{}", ENVVARS.neo4j_dns, ENVVARS.neo4j_port);
        loop {
            match Graph::new(&url, user, password).await {
                Err(err) => {
                    log::error!("failed to connect to log4j: {err}, retrying in a few secs");
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
                Ok(e) => break e,
            }
        }
    };
    log::info!("log4j inited");

    let kv_db = KvDb::new(&PathBuf::from("./db")).unwrap();
    let app_state_kv_db = kv_db.clone();
    let scraper_controller =
        Arc::new(load_scraper_controller(&kv_db, SCRAPER_CONTROLLER_KV_KEY).unwrap());
    scraper_controller.enable();

    let app_state = AppState {
        scraper_controller,
        proxy_settings_store: Arc::new(ProxySettingsStore::new(&ENVVARS.urls_file).unwrap()),
        shutdown: Arc::new(Shutdown::default()),
        kafka_outlet_producer: producer,
        kv_db: app_state_kv_db,
        proxy_connection_store: Arc::new(ProxyConnectionStore::default()),
        neo4j_pool,
        db_pool,
    };

    // create the public http server
    let public_addr = SocketAddr::new(
        ENVVARS.public_harvester_bind_address,
        ENVVARS.public_harvester_port,
    );
    let pub_app_state = app_state.clone();
    let pub_shutdown = app_state.shutdown.to_owned();
    tokio::spawn(async move {
        let public_listener = tokio::net::TcpListener::bind(public_addr).await.unwrap();
        let router = Router::new()
            .merge(public_endpoints(&pub_app_state))
            .with_state(pub_app_state.to_owned())
            .layer(TraceLayer::new_for_http())
            .layer(from_fn_with_state(pub_app_state, logging_middleware));

        info!("public server running on {public_addr}!");
        axum::serve(public_listener, router)
            .with_graceful_shutdown(shutdown_signal(pub_shutdown))
            .await
            .unwrap();
    });

    // create the pub http server
    let private_addr = SocketAddr::new(
        ENVVARS.private_harvester_bind_address,
        ENVVARS.private_harvester_port,
    );

    let priv_app_state = app_state.clone();
    let priv_shutdown = app_state.shutdown.to_owned();
    tokio::spawn(async move {
        let private_listener = tokio::net::TcpListener::bind(private_addr).await.unwrap();
        let router = Router::new()
            .merge(private_endpoints(&priv_app_state))
            .with_state(priv_app_state.to_owned())
            .layer(TraceLayer::new_for_http())
            .layer(from_fn_with_state(priv_app_state, logging_middleware));

        info!("private server running on {private_addr}!");
        axum::serve(private_listener, router)
            .with_graceful_shutdown(shutdown_signal(priv_shutdown))
            .await
            .unwrap();
    });

    // create a bg thread for saving the scraper state
    let app_state_save_thread = app_state.to_owned();
    tokio::spawn(async move {
        let app_state = app_state_save_thread;
        loop {
            if app_state.shutdown.task_tracker.is_closed() {
                break;
            }

            if let Err(err) = save_scraper_controller(
                &app_state.kv_db,
                &app_state.scraper_controller,
                SCRAPER_CONTROLLER_KV_KEY,
            ) {
                log::error!("failed to save scraper controller to kv: {err:?}");
            };

            app_state
                .scraper_controller
                .mark_dead_tasks(&chrono::Duration::minutes(5), &())
                .await;
            sleep(Duration::from_mins(5)).await;
        }
    });

    app_state.shutdown.task_tracker.wait().await;
    info!("shutting down!");
    save_scraper_controller(
        &kv_db,
        &app_state.scraper_controller,
        SCRAPER_CONTROLLER_KV_KEY,
    )
    .unwrap();

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
