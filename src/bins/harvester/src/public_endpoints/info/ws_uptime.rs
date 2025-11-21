use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::HeaderMap,
    response::Response,
};
use chrono::Utc;
use log::{debug, error, info, warn};
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

    let shutdown = app_state.shutdown.clone();
    Ok(ws.on_upgrade(move |e| async move {
        if shutdown.task_tracker.is_closed() {
            warn!(
                "Server is shutting down, refusing to accept new ws connection to ws/info/uptime"
            );
            return;
        }

        let handle = shutdown.task_tracker.spawn(async move {
            handle_socket(e, machine_id, app_state).await;
        });

        if let Err(e) = handle.await {
            if e.is_panic() {
                error!("ws/info/uptime panicked with error: {e}");
            } else {
                info!("ws/info/uptime closed: {e}");
            }
        };
    }))
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

    loop {
        tokio::select! {
            _ = app_state.shutdown.token.cancelled() => {
                info!("shutting down ws thread for machine id: {machine_id}");
                break;
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        if let Message::Ping(e) = msg {
                            debug!("device: {machine_id:?} pinged uptime ws");
                            if let Err(err) = socket.send(Message::Pong(e)).await {
                                info!("ws disconnected for machine id: {machine_id}, {err}");
                                break;
                            };
                        }
                    }
                    _ => break
                }
            }
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
    debug!("machine id: {machine_id} disconnected from ws/info/uptime");
}
