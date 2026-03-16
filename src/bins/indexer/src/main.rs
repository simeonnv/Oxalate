use std::{fmt, net::IpAddr, str::FromStr, time::Duration};

use axum::Router;
use envconfig::Envconfig;
use neo4rs::Graph;
use oxalate_env::load_env_vars;
use oxalate_init::{init_kafka_producer, init_logger, init_neo4j_pool, init_postgres_pool};
use rdkafka::producer::FutureProducer;
use sqlx::{Pool, Postgres};

pub mod endpoints;
pub mod scraping;

use tokio::time::sleep;
use tower_http::cors::{Any, Cors, CorsLayer};
use url::Url;
use wreq_util::Emulation;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub neo4j_pool: Graph,
    pub kafka_producer_client: Option<FutureProducer>,
    pub reqwest_client: reqwest::Client,
    pub wreq_client: wreq::Client,
    pub parser_url: Url,
    pub env_vars: &'static EnvVars,
}

#[derive(Envconfig)]
pub struct EnvVars {
    #[envconfig(from = "RUST_LOG", default = "info")]
    pub rust_log: String,

    // Kafka
    #[envconfig(from = "KAFKA_PORT", default = "19092")]
    pub kafka_port: u16,
    #[envconfig(from = "KAFKA_DNS")] // , default = "oxalate_redpanda"
    pub kafka_dns: Option<String>, // depending if this is none you can disable kafka logging
    #[envconfig(from = "KAFKA_MESSAGE_TIMEOUT_MS", default = "5000")]
    pub kafka_message_timeout_ms: u64,

    #[envconfig(from = "KAFKA_INDEXER_LOGS_TOPIC", default = "indexer_logs")]
    pub kafka_indexer_logs_topic: String,

    // Neo4j
    #[envconfig(from = "NEO4J_AUTH", default = "neo4j/rootrootroot")]
    pub neo4j_auth: String,
    #[envconfig(from = "NEO4J_PORT", default = "7687")]
    pub neo4j_port: u16,
    #[envconfig(from = "NEO4J_DNS")]
    pub neo4j_dns: String,

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

    #[envconfig(from = "PARSER_DNS", default = "oxalate-parser")]
    pub parser_dns: String,
    #[envconfig(from = "PARSER_PORT", default = "11167")]
    pub parser_port: u16,

    // Indexer
    #[envconfig(from = "INDEXER_BIND_ADDRESS", default = "0.0.0.0")]
    pub indexer_bind_address: IpAddr,
    #[envconfig(from = "INDEXER_PORT", default = "22267")]
    pub indexer_port: u16,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("AppState");
        let dbg = dbg.field("db_pool", &self.db_pool);

        let kpc_str = match &self.kafka_producer_client {
            Some(_) => "Some(<FutureProducer>)",
            None => "None",
        };
        dbg.field("kafka_producer_client", &kpc_str).finish()
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let env_vars: &'static EnvVars = load_env_vars();

    let producer: Option<FutureProducer> = match env_vars.kafka_dns.as_ref() {
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
        env_vars.kafka_indexer_logs_topic.to_owned(),
        producer.to_owned(),
    )
    .await;

    let db_pool = init_postgres_pool(
        &env_vars.postgres_user,
        &env_vars.postgres_password,
        &env_vars.db_dns,
        env_vars.db_port,
        &env_vars.postgres_db,
        env_vars.pool_max_conn,
    )
    .await;

    let neo4j_pool = init_neo4j_pool(
        &env_vars.neo4j_auth,
        &env_vars.neo4j_dns,
        env_vars.neo4j_port,
    )
    .await;

    let wreq_client = wreq::Client::builder()
        .local_address(IpAddr::from_str("0.0.0.0").unwrap())
        .emulation(Emulation::Firefox139)
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let reqwest_client = reqwest::Client::default();
    let parser_url = Url::parse(&format!(
        "http://{}:{}",
        &env_vars.parser_dns, env_vars.parser_port
    ))
    .unwrap();
    loop {
        match reqwest_client.head(parser_url.as_str()).send().await {
            Ok(_) => {
                log::info!("parser pinged successfully!");
                break;
            }
            Err(err) => {
                log::error!(
                    "failed to ping parser at {parser_url} with err: {err}, will retry again, blocking the program till it resolves"
                );
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        }
    }

    let state = AppState {
        db_pool,
        kafka_producer_client: producer,
        neo4j_pool,
        wreq_client,
        reqwest_client,
        env_vars,
        parser_url,
    };
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(endpoints::endpoints(&state))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        env_vars.indexer_bind_address, env_vars.indexer_port
    ))
    .await
    .unwrap();
    log::info!("server listening on {listener:?}");
    axum::serve(listener, app).await.unwrap();
}
