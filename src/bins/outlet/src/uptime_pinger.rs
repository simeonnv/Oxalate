use log::{error, info};
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

use crate::HARVESTER_URL;

pub fn uptime_pinger(reqwest_client: Client) {
    tokio::spawn(async move {
        let url = format!("http://{}/info/uptime", *HARVESTER_URL);
        loop {
            info!("pinging!");
            if let Err(err) = reqwest_client.get(&url).send().await {
                let status = err.status();
                error!("failed to send get request to /info/uptime: {err} status: {status:?}");
            };
            sleep(Duration::from_secs(10)).await;
        }
    });
}
