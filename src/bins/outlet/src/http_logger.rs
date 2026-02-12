use std::{io::Write, sync::Mutex};

use oxalate_schemas::harvester::public::info::post_logs::Req;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde_json::Value;
use tokio::{
    sync::mpsc::{Sender, UnboundedSender, channel, unbounded_channel},
    task::JoinHandle,
};

use crate::{HARVESTER_URL, MACHINE_ID};

const LOG_BUFFER_AMOUNT: usize = 32;

pub struct HttpLogger {
    http_client: Client,
    pub tx: UnboundedSender<Value>,
    handle: JoinHandle<()>,
}

impl Drop for HttpLogger {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl HttpLogger {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("machine-id", HeaderValue::from_str(&MACHINE_ID).unwrap());
        let http_client = Client::builder().default_headers(headers).build().unwrap();

        let (tx, mut rx) = unbounded_channel();

        let client = http_client.to_owned();
        let handle = tokio::spawn(async move {
            let mut buff = Vec::with_capacity(256);
            loop {
                let n = rx.recv_many(&mut buff, 128).await;
                if n == 0 {
                    break;
                }

                let logs = &buff[..];
                let req = Req {
                    logs: logs.to_vec(),
                };
                if let Err(err) = client
                    .post(format!("http://{}/info/logs", *HARVESTER_URL))
                    .json(&req)
                    .send()
                    .await
                {
                    eprintln!("failed to send logs: {err}");
                };

                buff.clear();
            }
        });

        Self {
            http_client,
            handle,
            tx,
        }
    }
}

impl Write for HttpLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let log: Value = match serde_json::from_slice(buf) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("failed to serialize buff into json for log: {e}");
                return Ok(0);
            }
        };
        if let Err(e) = self.tx.send(log) {
            eprintln!("Failed to send log to http logging thread: {}", e);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
