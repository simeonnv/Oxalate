use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use log::info;

use crate::AppState;

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
        None => return ().into_response(),
    };
    ws.on_upgrade(move |e| handle_socket(e, device_id, app_state))
}

async fn handle_socket(mut socket: WebSocket, device_id: Box<str>, app_state: AppState) {
    info!("proxy ws connected");
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(e) => e,
            Err(err) => {
                info!("proxy ws disconnected: {err}");
                return;
            }
        };

        match socket.send(msg).await {
            Err(err) => {
                info!("proxy ws disconnected: {err}");
                return;
            }
            _ => {}
        }
    }
}
