use std::time::Duration;

use log::{error, info};
use reqwest::Client;
use tokio::time::sleep;

use crate::HARVESTER_URL;

pub fn uptime_pinger(reqwest_client: Client) {
    tokio::spawn(async move {
        loop {
            info!("pinging /info/uptime");
            if let Err(err) = reqwest_client
                .get(format!("{}/info/uptime", *HARVESTER_URL))
                .send()
                .await
            {
                error!("failed to send request to /info/uptime! {err}");
            }
            sleep(Duration::from_mins(1)).await;
        }
    });
}
