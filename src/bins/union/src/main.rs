use axum::Router;
use kafka_writer_rs::KafkaLogWriter;
use log_json_serializer::parse_log;
use oxalate_env::ENVVARS;
use rdkafka::{ClientConfig, producer::FutureProducer};
use tower_http::cors::{self, Any};

#[derive(Clone)]
pub struct AppState {
    pub kafka_producer_client: Option<FutureProducer>,
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
