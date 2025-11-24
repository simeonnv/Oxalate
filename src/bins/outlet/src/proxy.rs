use std::time::Duration;

use log::error;
use reqwest::header::HeaderValue;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};

use crate::{HARVESTER_URL, MACHINE_ID, WsConnection};

pub async fn proxy() {
    tokio::spawn(async move {
        let url = format!("ws://{}/proxy", *HARVESTER_URL);
        let mut ws = WsConnection::connect(url).await;

        let nigga = ws.recv().await;
    });
    // tokio::spawn(async move {
    //     loop {
    //         let url = format!("ws://{}/proxy", *HARVESTER_URL);
    //         let header = HeaderValue::from_str(&MACHINE_ID).unwrap();

    //         let mut request = url.into_client_request().unwrap();
    //         request.headers_mut().insert("machine-id", header);

    //         let (ws_stream, response) = match connect_async(request).await {
    //             Ok(stream) => stream,
    //             Err(err) => {
    //                 error!("failed to connect to ws /proxy: {err}");
    //                 sleep(Duration::from_secs(15)).await;
    //                 continue;
    //             }
    //         };

    //         info!(
    //             "WebSocket handshake successful, status: {}",
    //             response.status()
    //         );
    //         let (mut ws, _) = ws_stream.split();

    //         loop {
    //             if let Err(err) = ws.send(Message::Ping(vec![].into())).await {
    //                 error!("disconnected from ws /info/uptime: {err}");
    //                 break;
    //             }
    //             info!("Ping sent to uptime ws");
    //             sleep(Duration::from_secs(15)).await;
    //         }
    //     }
    // });
}
