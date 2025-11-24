use axum::{Json, extract::State, http::HeaderMap};
use uuid::Uuid;

use oxalate_schemas::harvester::public::info::post_logs::*;

use crate::{AppState, Error, insure_device_exists};

#[utoipa::path(
    post,
    path = "/info/logs",
    request_body = Req,
    responses(),
    params(
      ("machine-id" = String, Header, description = "hardware uuid"),
    ),
    tag = "Info",
)]
#[axum::debug_handler]
pub async fn post_logs(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), Error> {
    let machine_id = headers.get("machine-id").and_then(|v| v.to_str().ok());
    let machine_id = match machine_id {
        Some(e) => e,
        None => return Err(Error::BadRequest("no machine-id header!".into())),
    };
    insure_device_exists(machine_id, &app_state.db_pool).await?;

    for log in &req.logs {
        let db_pool = app_state.db_pool.clone();
        let id = Uuid::new_v4();
        sqlx::query!(
            "
                INSERT INTO Logs
                    (id, log_level, body, device_machine_id)
                VALUES ($1, $2, $3, $4);
            ",
            id,
            log.log_level,
            log.body,
            machine_id,
        )
        .execute(&db_pool)
        .await?;
    }

    Ok(())
}
