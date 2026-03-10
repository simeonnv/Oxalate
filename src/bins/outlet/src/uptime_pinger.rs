use crate::AppState;
use futures::{SinkExt, StreamExt};
use reqwest::header::HeaderValue;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, handshake::client::generate_key, http::Request},
};
use tokio_util::bytes::Bytes;

pub fn uptime_pinger(global_state: AppState) {
    tokio::spawn(async move {
        let url = format!(
            "ws://{}:{}/info/uptime",
            global_state.env_vars.harvester_dns, global_state.env_vars.public_harvester_port
        );
        let request = Request::builder()
            .uri(&url)
            .header("Host", global_state.env_vars.harvester_dns.to_owned())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .header(
                "machine-id",
                HeaderValue::from_str(global_state.machine_id).unwrap(),
            )
            .body(())
            .expect("failed to build static uptime pinger request body");

        loop {
            let ws_stream = match connect_async(request.to_owned()).await {
                Ok((stream, _)) => {
                    log::info!("Uptime ws connected");
                    stream
                }
                Err(e) => {
                    log::error!("failed to connect to uptime ws, will retry later: {e}");
                    sleep(Duration::from_secs(60)).await;
                    continue;
                }
            };

            let (mut sender, _) = ws_stream.split();

            loop {
                if sender
                    .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
                    .await
                    .is_err()
                {
                    log::error!("uptime ws connection died: retrying");
                    break;
                }
                log::info!("uptime ws pinged");
                sleep(Duration::from_secs(30)).await;
            }
        }
    });
}
