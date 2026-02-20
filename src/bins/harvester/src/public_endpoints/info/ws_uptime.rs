use std::time::Duration;

use crate::{AppState, middleware::logging_middleware::LoggingCTX};
use axum::{
    Extension,
    body::Bytes,
    debug_handler,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
};
use exn::ResultExt;
use http_error::HttpError;
use oxalate_scraper_controller::ProxyId;
use tokio::time::sleep;

#[utoipa::path(
    get,
    path = "/info/uptime",
    responses(
        (status = 200),
    ),
    description = "WS ENDPOINT",
    params(
      ("machine-id" = String, Header, description = "Device hardware id"),
    ),
    tag = "Info",
)]
#[debug_handler]
pub async fn ws_uptime(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(proxy_id): Extension<ProxyId>,
    Extension(logging_ctx): Extension<LoggingCTX>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_uptime_task(socket, state, proxy_id, logging_ctx))
}

pub async fn ws_uptime_task(
    mut socket: WebSocket,
    app_state: AppState,
    proxy_id: ProxyId,
    _logging_ctx: LoggingCTX,
) {
    app_state
        .proxy_connection_store
        .connected(proxy_id.to_owned())
        .await;

    loop {
        sleep(Duration::from_secs(5)).await;
        if socket
            .send(axum::extract::ws::Message::Ping(Bytes::from_static(&[
                1, 2, 3,
            ])))
            .await
            .is_err()
        {
            app_state
                .proxy_connection_store
                .disconnected(proxy_id)
                .await;
            break;
        }

        if let Err(err) = sqlx::query!(
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
        .or_raise(|| HttpError::Internal("".into()))
        {
            log::error!("ws_uptime shutdowned unexpectedly: {err:?}");
            app_state
                .proxy_connection_store
                .disconnected(proxy_id)
                .await;
            break;
        };
    }
}
