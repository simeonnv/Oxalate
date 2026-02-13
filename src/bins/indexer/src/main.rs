use std::fmt;

use axum::Router;
use rdkafka::{ClientConfig, producer::FutureProducer};
use sqlx::{Pool, Postgres};

use crate::env::ENVVARS;

pub mod endpoints;

pub mod env;

mod create_postgres_pool;
pub use create_postgres_pool::create_postgres_pool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub kafka_producer_client: Option<FutureProducer>,
};

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("db_pool", &self.db_pool)
            .field("kafka_producer_client", &"ti si gei")
            .finish()
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let _ = ENVVARS.rust_log;

    let client = ENVVARS.kafka_address.map(|e| {
        ClientConfig::new()
            .set("bootstrap.servers", format!("{e}:{}", ENVVARS.kafka_port))
            .set(
                "message.timeout.ms",
                ENVVARS.kafka_message_timeout_ms.to_string(),
            )
            .create()
            .expect("failed to create kafka client")
    });


    let db_pool = create_postgres_pool(
        &ENVVARS.postgres_user,
        &ENVVARS.postgres_password,
        &ENVVARS.db_address,
        ENVVARS.db_port,
        &ENVVARS.postgres_name,
        ENVVARS.pool_max_conn,
    )
    .await
    .expect("failed to connect to db");

    let state = AppState { db_pool, kafka_producer_client: client };

    let app = Router::new()
        .merge(endpoints::endpoints(&state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        ENVVARS.indexer_address, ENVVARS.indexer_port
    ))
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap();
}
