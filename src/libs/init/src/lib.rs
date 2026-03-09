use std::time::Duration;

use exn::{Result, ResultExt};
use kafka_writer_rs::KafkaLogWriter;
use log_json_serializer::parse_log;
use neo4rs::Graph;
use rdkafka::{ClientConfig, producer::FutureProducer};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::time::sleep;

#[derive(thiserror::Error, Debug)]
pub enum KafkaError {
    #[error("Failet to connect to Kafka to create producer")]
    Connecton,
}

pub async fn init_kafka_producer(
    kafka_dns: &str,
    kafka_port: u16,
    msg_timeout: u64,
) -> Result<FutureProducer, KafkaError> {
    let kafka_connect_url = format!("{}:{}", kafka_dns, kafka_port);
    println!("kafka connect url is: {kafka_connect_url}");

    let client = ClientConfig::new()
        .set("bootstrap.servers", kafka_connect_url)
        .set("message.timeout.ms", msg_timeout.to_string())
        .create()
        .or_raise(|| KafkaError::Connecton)?;

    println!("connected to kafka and inited kafka future producer");

    Ok(client)
}

pub async fn init_logger(logs_topic: String, producer: Option<FutureProducer>) {
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
                    KafkaLogWriter::new(client.to_owned(), Box::leak(Box::new(logs_topic))).await,
                );

                fern.chain(fern::Output::writer(kafka_writer, "\n"))
            }
            _ => fern,
        }
    };
    fern.apply().expect("failed to create logger");
    log::info!("inited logger");
}

pub async fn init_postgres_pool(
    postgres_user: &str,
    postgres_password: &str,
    db_dns: &str,
    db_port: u16,
    postgres_db: &str,
    max_conn: u32,
) -> Pool<Postgres> {
    let db_url: String = format!(
        "postgres://{}:{}@{}:{}/{}",
        postgres_user, postgres_password, db_dns, db_port, postgres_db
    );
    log::info!("creating a connection with db: {}", postgres_db);
    log::info!("postgres connection url: {db_url}");

    let pool = loop {
        let pool = PgPoolOptions::new()
            .max_connections(max_conn)
            .connect(&db_url)
            .await;
        match pool {
            Ok(e) => {
                break e;
            }
            Err(err) => {
                eprintln!("failed to connect to postres: {err}, retrying in a few secs");
                sleep(Duration::from_secs(30)).await;
                continue;
            }
        };
    };
    log::info!("connected to the db successfully");

    pool
}

pub async fn init_neo4j_pool(neo4j_auth: &str, neo4j_dns: &str, neo4j_port: u16) -> Graph {
    let neo4j_pool = {
        let mut parts = neo4j_auth.split("/");
        let user = parts.next().unwrap_or("root");
        let password = parts.next().unwrap_or("root");
        let url = format!("{}:{}", neo4j_dns, neo4j_port);
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

    neo4j_pool
}
