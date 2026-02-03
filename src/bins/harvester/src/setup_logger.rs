use std::collections::BTreeMap;
use std::fmt::Arguments;

use chrono::Utc;
use exn::Exn;
use fern::FormatCallback;
use log::Record;
use log::kv::Key;
use log::kv::Value;
use log::kv::VisitSource;
use rdkafka::{ClientConfig, producer::FutureProducer};

use crate::env::ENVVARS;
use crate::kafka_logging_writer::KafkaLogWriter;

use exn::ResultExt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to setup kafka producer")]
    Producer,
    #[error("failed to create fern logger")]
    Fern,
}

pub async fn setup_logger() -> Result<(), Exn<Error>> {
    let fern = fern::Dispatch::new()
        .format(log_formatter)
        .level(log::LevelFilter::Debug)
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

struct Collect<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);
impl<'kvs> VisitSource<'kvs> for Collect<'kvs> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), log::kv::Error> {
        self.0.insert(key, value);
        Ok(())
    }
}

fn log_formatter(out: FormatCallback, message: &Arguments, record: &Record) {
    let mut visitor = Collect(BTreeMap::new());
    record.key_values().visit(&mut visitor).unwrap();
    let kv = {
        let mut kv = visitor.0;
        kv.insert(
            Key::from("msg"),
            if let Some(msg) = message.as_str() {
                Value::from(msg)
            } else {
                Value::from_display(message)
            },
        );

        kv.insert(Key::from_str("target"), Value::from(record.target()));

        if let Some(val) = record.module_path() {
            kv.insert(Key::from("module"), Value::from(val));
        }
        if let Some(val) = record.file() {
            kv.insert(Key::from("file"), Value::from(val));
        }
        if let Some(val) = record.line() {
            kv.insert(Key::from("line"), Value::from(val));
        }

        let now: i64 = Utc::now().timestamp();
        kv.insert(Key::from("timestamp"), Value::from(now));

        kv
    };
    let json = serde_json::to_string(&kv).unwrap();

    out.finish(format_args!("{}", json))
}
