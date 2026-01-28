use std::{collections::BTreeMap, io::Write, time::Duration};

use log::kv::{Key, Value};
use rdkafka::producer::{FutureProducer, FutureRecord};
use structured_logger::Writer;
use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};

pub struct KafkaLoggerWriter {
    thread_handle: JoinHandle<()>,
    log_tx: Sender<Box<[u8]>>,
}

impl KafkaLoggerWriter {
    pub fn new(kafka_client: FutureProducer, topic: &'static str) -> Self {
        let (tx, mut rx) = mpsc::channel::<Box<[u8]>>(512);
        let handle = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let status = kafka_client
                    .send(
                        FutureRecord::to(topic)
                            .payload(data.as_ref())
                            .key(&format!("key-{}", uuid::Uuid::new_v4())),
                        Duration::from_secs(0),
                    )
                    .await;
                let _ = dbg!(status);
            }
        });

        Self {
            thread_handle: handle,
            log_tx: tx,
        }
    }
}

impl Writer for KafkaLoggerWriter {
    fn write_log(&self, value: &BTreeMap<Key, Value>) -> Result<(), std::io::Error> {
        let mut buf = Vec::with_capacity(256);
        serde_json::to_writer(&mut buf, value).map_err(std::io::Error::from)?;
        buf.write_all(b"\n")?;

        self.log_tx.blocking_send(buf.into_boxed_slice()).unwrap();

        Ok(())
    }
}
