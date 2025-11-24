use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use log::error;
use reqwest::header::HeaderValue;
use tokio::{net::TcpStream, time::sleep};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

use crate::MACHINE_ID;

pub struct WsConnection {
    url: String,
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsConnection {
    pub async fn send(&mut self, message: Message) {
        loop {
            match self.ws.send(message.clone()).await {
                Ok(_) => break,
                Err(err) => {
                    error!("disconnected from proxy ws, reconnecting: {err}");
                    sleep(Duration::from_secs(30)).await;
                    *self = Self::connect(self.url.to_owned()).await;
                }
            }
        }
    }

    pub async fn recv(&mut self) -> Message {
        loop {
            match self.ws.next().await {
                Some(Ok(msg)) => return msg,
                Some(Err(err)) => {
                    error!("error recieving proxy ws, reconnecting: {err}");
                    sleep(Duration::from_secs(30)).await;
                    *self = Self::connect(self.url.to_owned()).await;
                }
                None => {
                    error!("disconnected from proxy ws, reconnecting");
                    sleep(Duration::from_secs(30)).await;
                    *self = Self::connect(self.url.to_owned()).await;
                }
            };
        }
    }

    pub async fn connect(url: String) -> Self {
        let header = HeaderValue::from_str(&MACHINE_ID).unwrap();
        let mut request = url.as_str().into_client_request().unwrap();
        request.headers_mut().insert("machine-id", header);

        let ws = loop {
            let (ws, _) = match connect_async(request.to_owned()).await {
                Ok(e) => e,
                Err(err) => {
                    error!("failed to connect to proxy ws, retrying! {err}");
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            // let status = res.status();
            // if status != 200 || status != 101 {
            //     error!("failed to connect to proxy ws with response: {status}");
            //     sleep(Duration::from_secs(30)).await;
            //     continue;
            // }

            break ws;
        };

        Self { url, ws }
    }
}
