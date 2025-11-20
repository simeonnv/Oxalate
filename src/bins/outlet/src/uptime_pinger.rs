use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use reqwest::header::HeaderValue;
use std::{borrow::Cow, fmt::Debug, time::Duration};
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::Request, protocol::WebSocketConfig},
};

use crate::{HARVESTER_URL, MACHINE_ID};

pub fn uptime_pinger() {
    tokio::spawn(async move {
        loop {
            let url = format!("ws://{}/info/uptime", *HARVESTER_URL);
            let header = HeaderValue::from_str(&MACHINE_ID).unwrap();

            let mut request = url.into_client_request().unwrap();
            request.headers_mut().insert("machine-id", header);

            let (ws_stream, response) = match connect_async(request).await {
                Ok(stream) => stream,
                Err(err) => {
                    error!("failed to connect to ws /info/uptime: {err}");
                    sleep(Duration::from_secs(15)).await;
                    continue;
                }
            };

            info!(
                "WebSocket handshake successful, status: {}",
                response.status()
            );
            let (mut ws, _) = ws_stream.split();

            loop {
                if let Err(err) = ws.send(Message::Ping(vec![].into())).await {
                    error!("disconnected from ws /info/uptime: {err}");
                    break;
                }
                info!("Ping sent to uptime ws");
                sleep(Duration::from_secs(15)).await;
            }
        }
    });
}
