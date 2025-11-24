use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use reqwest::header::HeaderValue;
use std::{borrow::Cow, fmt::Debug, time::Duration};
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::Request, protocol::WebSocketConfig},
};

use crate::{HARVESTER_URL, MACHINE_ID, WsConnection};

pub fn uptime_pinger() {
    tokio::spawn(async move {
        let url = format!("ws://{}/info/uptime", *HARVESTER_URL);
        let mut ws = WsConnection::connect(url).await;

        loop {
            ws.send(Message::Ping(vec![].into())).await;
            info!("Ping sent to uptime ws");
            sleep(Duration::from_secs(15)).await;
        }
    });
}
