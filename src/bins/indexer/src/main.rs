use std::{fmt, time::Duration};

use axum::Router;
use kafka_writer_rs::KafkaLogWriter;
use log_json_serializer::parse_log;
use neo4rs::Graph;
use oxalate_env::ENVVARS;
use rdkafka::{ClientConfig, producer::FutureProducer};
use sqlx::{Pool, Postgres};
use tokio::time::sleep;
use tower_http::cors::{self, Any};

pub mod endpoints;
pub mod scraping;

mod create_postgres_pool;

pub use create_postgres_pool::create_postgres_pool;
use wreq::Client;
use wreq_util::Emulation;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub neo4j_pool: Graph,
    pub kafka_producer_client: Option<FutureProducer>,
    pub wreqclient: wreq::Client,
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

    let client: Option<FutureProducer> = ENVVARS.kafka_dns.as_ref().map(|dns| {
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
        match client {
            Some(ref client) => {
                let kafka_writer = Box::new(
                    KafkaLogWriter::new(client.to_owned(), &ENVVARS.kafka_indexer_logs_topic).await,
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
    .expect("failed to connect to db");

    let neo4j_pool = {
        let mut parts = ENVVARS.neo4j_auth.split("/");
        let user = parts.next().unwrap_or("root");
        let password = parts.next().unwrap_or("root");
        let url = format!("{}:{}", ENVVARS.neo4j_bind_address, ENVVARS.neo4j_port);
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
    log::info!("neo4j inited");

    let wreqclient = Client::builder()
        .emulation(Emulation::Firefox139)
        .build()
        .unwrap();

    let state = AppState {
        db_pool,
        kafka_producer_client: client,
        neo4j_pool,
        wreqclient,
    };

    // DEVELOPMENT ONLY
    let cors = cors::CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let thingy = scraping::google::request("hitler", &state).await.unwrap();
    let goy = scraping::google::parse_response(&thingy);
    dbg!(goy);

    let app = Router::new()
        .merge(endpoints::endpoints(&state))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        ENVVARS.indexer_bind_address, ENVVARS.indexer_port
    ))
    .await
    .unwrap();
    log::info!("server listening on {listener:?}");
    axum::serve(listener, app).await.unwrap();
}
