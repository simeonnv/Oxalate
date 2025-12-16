use std::sync::Mutex;

use chrono::Utc;
use log::{LevelFilter, Log, SetLoggerError};
use oxalate_schemas::harvester::public::info::post_logs::{Log as ReqLog, Req};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};

use crate::{HARVESTER_URL, MACHINE_ID};

const LOG_BUFFER_AMOUNT: usize = 1;

pub struct HttpLogger {
    log_level: LevelFilter,
    log_buffer: Mutex<Vec<ReqLog>>,
    http_client: Client,
}

impl Log for HttpLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let now = Utc::now().naive_local();
        let level = record.level();
        let file = record.file().unwrap_or("").to_owned();
        let row = record.line().unwrap_or(0);
        let body = record.args().to_string();

        let req_log = ReqLog {
            log_level: level.to_string(),
            time: now,
            file,
            row,
            body,
        };

        if let Ok(mut buffer) = self.log_buffer.lock() {
            buffer.push(req_log);
            if buffer.len() >= LOG_BUFFER_AMOUNT {
                let buff_clone = buffer.to_owned();
                buffer.clear();
                drop(buffer);
                let http_client = self.http_client.to_owned();
                send_logs(buff_clone, http_client);
            }
        }
    }

    fn flush(&self) {
        let logs = match self.log_buffer.lock() {
            Ok(e) => e.to_owned(),
            Err(err) => {
                eprintln!("failed to flush logs! {err}");
                return;
            }
        };
        let http_client = self.http_client.to_owned();
        send_logs(logs, http_client);
    }
}

impl HttpLogger {
    pub fn new(log_level: LevelFilter) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("machine-id", HeaderValue::from_str(&MACHINE_ID).unwrap());
        let client = Client::builder().default_headers(headers).build().unwrap();

        Self {
            log_level,
            log_buffer: Mutex::new(Vec::with_capacity(LOG_BUFFER_AMOUNT)),
            http_client: client,
        }
    }
}

pub fn init_http_logger(log_level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(HttpLogger::new(log_level)))
        .map(|()| log::set_max_level(log_level))
}

fn send_logs(buff: Vec<ReqLog>, http_client: Client) {
    tokio::spawn(async move {
        let req = Req { logs: buff };
        if let Err(err) = http_client
            .post(format!("http://{}/info/logs", *HARVESTER_URL))
            .json(&req)
            .send()
            .await
        {
            eprintln!("failed to send logs: {err}");
        };
    });
}
