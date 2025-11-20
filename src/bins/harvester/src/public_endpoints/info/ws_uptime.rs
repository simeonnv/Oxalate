use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::HeaderMap,
    response::Response,
};
use chrono::Utc;
use log::{error, info};
use uuid::Uuid;

use crate::{AppState, Error, insure_device_exists};

#[utoipa::path(
    get,
    path = "/proxy",
    responses(
        (status = 200),
    ),
    description = "",
    params(
      ("machine-id" = String, Header, description = "Device hardware id"),
    ),
    tag = "Proxy",
)]
pub async fn ws_uptime(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<Response, Error> {
    let machine_id = headers.get("machine-id").and_then(|v| v.to_str().ok());
    let machine_id = match machine_id {
        Some(e) => e.to_owned(),
        None => return Err(Error::BadRequest("no or invalid machine id!".into())),
    };
    insure_device_exists(&machine_id, &app_state.db_pool).await?;
    Ok(ws.on_upgrade(move |e| handle_socket(e, machine_id, app_state)))
}

async fn handle_socket(mut socket: WebSocket, machine_id: String, app_state: AppState) {
    info!("{machine_id} connected to the proxy ws");
    app_state
        .uptime_connected_devices
        .insert(machine_id.clone(), Utc::now().naive_utc());

    let id = Uuid::new_v4();
    if let Err(err) = sqlx::query!(
        "
            INSERT INTO Uptime
                (id, device_machine_id, state)
            VALUES
                ($1, $2, 'connected')
            ;
        ",
        id,
        machine_id,
    )
    .execute(&app_state.db_pool)
    .await
    {
        error!("failed to insert connection uptime in database: {err}");
        return;
    };

    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Ping(e) = msg {
            info!("device: {machine_id:?} pinged uptime ws");
            let _ = socket.send(Message::Pong(e)).await;
        }
    }
    app_state.uptime_connected_devices.remove(&machine_id);

    let id = Uuid::new_v4();
    if let Err(err) = sqlx::query!(
        "
            INSERT INTO Uptime
                (id, device_machine_id, state)
            VALUES
                ($1, $2, 'disconnected')
            ;
        ",
        id,
        machine_id,
    )
    .execute(&app_state.db_pool)
    .await
    {
        error!("failed to insert disconnection uptime in database: {err}");
    };
}
