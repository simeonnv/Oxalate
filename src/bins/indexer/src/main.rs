use std::fmt;

use axum::Router;
use kafka_writer_rs::KafkaLogWriter;
use log_json_serializer::parse_log;
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
    let _ = ENVVARS.rust_log;

    let client: Option<FutureProducer> = ENVVARS.kafka_address.map(|e| {
        let client = ClientConfig::new()
            .set("bootstrap.servers", format!("{e}:{}", ENVVARS.kafka_port))
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
        match client {
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
        &ENVVARS.db_address,
        ENVVARS.db_port,
        &ENVVARS.postgres_name,
        ENVVARS.pool_max_conn,
    )
    .await
    .expect("failed to connect to db");

    let state = AppState {
        db_pool,
        kafka_producer_client: client,
    };

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
