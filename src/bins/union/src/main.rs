use axum::Router;
use kafka_writer_rs::KafkaLogWriter;
use log_json_serializer::parse_log;
use oxalate_env::ENVVARS;
use rdkafka::{ClientConfig, producer::FutureProducer};
use tower_http::cors::{self, Any};

use crate::union_db::UnionDB;

pub mod union_db;

#[derive(Clone)]
pub struct AppState {
    pub kafka_producer_client: Option<FutureProducer>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let union_db = UnionDB::<String>::new("./gei".into());
    let sex = String::from(
        "please enable cookies sorry you have been blocked you are unable access com why have i been blocked this website using security service protect itself from online attacks action you just performed triggered security solution there are several actions that could trigger this block including submitting certain word phrase sql command malformed data what can i do resolve this you can email site owner let them know you were blocked please include what you were doing when this page came up cloudflare ray id found bottom this page cloudflare ray id d ad f ee your ip click reveal performance security cloudflare",
    ).split(" ").map(|e| e.to_owned()).collect::<Vec<String>>();

    union_db.insert_buf(&sex, 1, 10);
    dbg!(union_db.extract_relations(&String::from("you"), 10));
    // dbg!(union_db);

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
                    KafkaLogWriter::new(client.to_owned(), &ENVVARS.kafka_union_logs_topic).await,
                );

                fern.chain(fern::Output::writer(kafka_writer, "\n"))
            }
            _ => fern,
        }
    };
    fern.apply().expect("failed to create logger");
    log::info!("inited logger");

    // let db_pool = create_postgres_pool(
    //     &ENVVARS.postgres_user,
    //     &ENVVARS.postgres_password,
    //     &ENVVARS.db_dns,
    //     ENVVARS.db_port,
    //     &ENVVARS.postgres_name,
    //     ENVVARS.pool_max_conn,
    // )
    // .await
    // .expect("failed to connect to db");

    let state = AppState {
        kafka_producer_client: client,
    };

    // DEVELOPMENT ONLY
    let cors = cors::CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        // .merge(endpoints::endpoints(&state))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        ENVVARS.union_bind_address, ENVVARS.union_port
    ))
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap();
}
