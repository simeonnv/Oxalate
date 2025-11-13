use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use log::{error, info};

use crate::{AppState, Error, scrapper_state::ProxyOutput};

#[utoipa::path(
    get,
    path = "/proxy",
    responses(
        (status = 200),
    ),
    params(
      ("device-id" = String, Header, description = "Device id"),
    ),
    tag = "Proxy",
)]
pub async fn ws_proxy(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    let device_id = headers.get("device-id").and_then(|v| v.to_str().ok());
    let device_id = match device_id {
        Some(e) => Box::from(e),
        None => return Error::BadRequest("no or invalid device id!".into()).into_response(),
    };
    ws.on_upgrade(move |e| handle_socket(e, device_id, app_state))
}

enum RequestType {
    RequestUrls,
    ReturnUrlOutputs,
}

impl RequestType {
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::RequestUrls),
            1 => Some(Self::ReturnUrlOutputs),
            _ => None,
        }
    }
}

async fn handle_socket(mut socket: WebSocket, device_id: Box<str>, mut app_state: AppState) {
    info!("{device_id} connected to the proxy ws");
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(e) => e,
            Err(err) => {
                info!("{device_id} disconnected to the proxy ws: {err}");
                return;
            }
        };
        let msg = msg.into_data();
        if msg.is_empty() || msg.len() < 2 {
            continue;
        }
        let req_type = match RequestType::from_byte(msg[0]) {
            Some(e) => e,
            None => {
                let _ = socket
                    .send(Message::Text("invalid request type!".into()))
                    .await;
                continue;
            }
        };
        let msg = &msg[1..];
        match req_type {
            RequestType::ReturnUrlOutputs => {
                let proxy_output: Vec<ProxyOutput> = match serde_json::from_slice(msg) {
                    Ok(e) => e,
                    Err(err) => {
                        let _ = socket.send(Message::Text(err.to_string().into())).await;
                        continue;
                    }
                };
                let db_result = app_state
                    .scrapper_state
                    .complete_job(
                        &device_id,
                        proxy_output.into_boxed_slice(),
                        app_state.db_pool.to_owned(),
                    )
                    .await;
                if let Err(err) = db_result {
                    #[cfg(debug_assertions)]
                    let _ = socket.send(Message::Text(err.to_string().into())).await;
                    error!("db error at completing job: {}", err);
                }
                continue;
            }
            RequestType::RequestUrls => {
                let job = app_state.scrapper_state.req_addresses(&device_id);
                let job = match job {
                    Some(e) => e,
                    None => continue,
                };
                let jobs_json = serde_json::to_string(&job.urls).unwrap();
                let _ = socket.send(Message::Text(jobs_json.into())).await;
            }
        }
    }
}
