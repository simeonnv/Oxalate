use log::error;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

use crate::HARVESTER_URL;

pub fn uptime_pinger(reqwest_client: Client) {
    tokio::spawn(async move {
        let url = format!("{}/info/uptime", *HARVESTER_URL);
        loop {
            if let Err(err) = reqwest_client.get(&url).send().await {
                error!("failed to send get request to /info/uptime!: {err}");
            };
            sleep(Duration::from_secs(60)).await;
        }
    });
}
