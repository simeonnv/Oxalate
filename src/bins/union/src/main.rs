use std::time::Duration;

use axum::Router;
use clap::{Parser, arg, command};
use kafka_writer_rs::KafkaLogWriter;
use log_json_serializer::parse_log;
use neo4rs::{Graph, query};
use oxalate_env::ENVVARS;
use rdkafka::{ClientConfig, producer::FutureProducer};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::time::sleep;
use tower_http::cors::{self, Any};

mod migrate_postgres_to_neo4j;
pub use migrate_postgres_to_neo4j::migrate_postgres_to_neo4j;

pub mod endpoints;

#[derive(Clone)]
pub struct AppState {
    pub kafka_producer_client: Option<FutureProducer>,
    pub log4j_pool: Graph,
    pub db_pool: Pool<Postgres>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = false)]
    migrate_postgres_to_neo4j: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let _ = ENVVARS.rust_log;
    let args = Args::parse();

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
                    KafkaLogWriter::new(client.to_owned(), &ENVVARS.kafka_union_logs_topic).await,
                );

                fern.chain(fern::Output::writer(kafka_writer, "\n"))
            }
            _ => fern,
        }
    };
    fern.apply().expect("failed to create logger");
    log::info!("inited logger");

    let db_pool = {
        let db_url: String = format!(
            "postgres://{}:{}@{}:{}/{}",
            ENVVARS.postgres_user,
            ENVVARS.postgres_password,
            ENVVARS.db_dns,
            ENVVARS.db_port,
            ENVVARS.postgres_db
        );
        log::info!("creating a connection with db: {}", ENVVARS.postgres_db);
        log::info!("postgres connection url: {db_url}");

        loop {
            let pool = PgPoolOptions::new()
                .max_connections(ENVVARS.pool_max_conn)
                .connect(&db_url)
                .await;
            match pool {
                Ok(e) => {
                    break e;
                }
                Err(err) => {
                    log::error!("failed to connect to postres: {err}, retrying in a few secs");
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };
        }
    };
    log::info!("postgres inited");

    let graph_db = {
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
    log::info!("log4j inited");

    graph_db
        .run(query(
            "
           CREATE CONSTRAINT IF NOT EXISTS FOR (w:Word) REQUIRE w.text IS UNIQUE;
        ",
        ))
        .await
        .unwrap();
    graph_db
        .run(query(
            "
           CREATE CONSTRAINT IF NOT EXISTS FOR (w:Website) REQUIRE w.url IS UNIQUE;
        ",
        ))
        .await
        .unwrap();

    let state = AppState {
        kafka_producer_client: client,
        log4j_pool: graph_db,
        db_pool,
    };

    if args.migrate_postgres_to_neo4j {
        log::info!("MIGRATING POSTGRES TO NEO4J");
        migrate_postgres_to_neo4j(state.to_owned()).await;
        log::info!("MIGRATING POSTGRES TO NEO4J IS DONE");
    }

    // DEVELOPMENT ONLY
    let cors = cors::CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(endpoints::endpoints(&state))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        ENVVARS.union_bind_address, ENVVARS.union_port
    ))
    .await
    .unwrap();
    log::info!(
        "oxalate_union running at {}:{}",
        ENVVARS.union_bind_address,
        ENVVARS.union_port,
    );
    axum::serve(listener, app).await.unwrap();
}
