use axum::{Json, extract::State, http::HeaderMap};
use oxalate_scrapper_controller::ProxyId;
use uuid::Uuid;

use oxalate_schemas::harvester::public::info::post_logs::*;

use crate::{AppState, Error};

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
    let proxy_id = ProxyId::from_http_headers(&headers, &app_state.db_pool).await?;

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
            proxy_id.as_ref(),
        )
        .execute(&db_pool)
        .await?;
    }

    Ok(())
}
