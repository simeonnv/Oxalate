use std::net::SocketAddr;

use crate::{AppState, middleware::logging_middleware::LoggingCTX};
use axum::{
    Extension,
    extract::{ConnectInfo, State, WebSocketUpgrade, ws::WebSocket},
    http::HeaderMap,
    response::IntoResponse,
};
use exn::ResultExt;
use http_error::HttpError;
use oxalate_scraper_controller::ProxyId;

#[utoipa::path(
    get,
    path = "/info/uptime",
    responses(
        (status = 200),
    ),
    description = "",
    params(
      ("machine-id" = String, Header, description = "Device hardware id"),
    ),
    tag = "Info",
)]
pub async fn get_uptime(
    headers: HeaderMap,
    State(app_state): State<AppState>,
) -> Result<(), HttpError> {
    let proxy_id = ProxyId::from_http_headers(&headers, &app_state.db_pool)
        .await
        .or_raise(|| HttpError::BadRequest("No proxy id in header".into()))?;

    sqlx::query!(
        "
            INSERT INTO Uptime
                (device_machine_id)
            VALUES
                ($1)
            ;
        ",
        proxy_id.as_ref()
    )
    .execute(&app_state.db_pool)
    .await
    .or_raise(|| HttpError::Internal("".into()))?;

    Ok(())
}

pub async fn ws_uptime(
    ws: WebSocketUpgrade,
    Extension(proxy_id): Extension<ProxyId>,
    Extension(logging_ctx): Extension<LoggingCTX>,
    // ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_uptime_task(socket, logging_ctx))
}

pub async fn ws_uptime_task(mut socket: WebSocket, logging_ctx: LoggingCTX) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(e) => e,
            _ => return,
        };
    }
}
