use exn::Exn;
use rdkafka::{ClientConfig, producer::FutureProducer};

use crate::env::ENVVARS;
use crate::kafka_logging_writer::KafkaLogWriter;

use log_json_serializer::parse_log;

use exn::ResultExt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to setup kafka producer while setting up logger")]
    Producer,
    #[error("failed to create fern logger")]
    Fern,
}

pub async fn setup_logger() -> Result<(), Exn<Error>> {
    let fern = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}",
                parse_log(message, record).expect("failed to serialize log into json")
            ));
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    let fern = match ENVVARS.kafka_address {
        Some(e) => {
            let fern = fern;
            let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers", format!("{e}:{}", ENVVARS.kafka_port))
                .set(
                    "message.timeout.ms",
                    ENVVARS.kafka_message_timeout_ms.to_string(),
                )
                .create()
                .or_raise(|| Error::Producer)?;
            let kafka_writer =
                Box::new(KafkaLogWriter::new(producer, &ENVVARS.kafka_harvester_logs_topic).await);

            fern.chain(fern::Output::writer(kafka_writer, "\n"))
        }
        None => fern,
    };

    fern.apply().or_raise(|| Error::Fern)?;

    Ok(())
}
