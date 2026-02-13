use std::time::Duration;

use axum::{Extension, Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;
use log::error;
use oxalate_scraper_controller::ProxyId;
use rdkafka::producer::FutureRecord;
use uuid::Uuid;

use oxalate_schemas::harvester::public::info::post_logs::*;

use crate::AppState;

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
    Extension(proxy_id): Extension<ProxyId>,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), HttpError> {
    for log in &req.logs {
        let db_pool = app_state.db_pool.clone();
        let id = Uuid::new_v4();
        sqlx::query!(
            "
                INSERT INTO Logs
                    (id, log, device_machine_id)
                VALUES ($1, $2, $3);
            ",
            id,
            log,
            proxy_id.as_ref(),
        )
        .execute(&db_pool)
        .await
        .or_raise(|| HttpError::Internal("".into()))?;

        // TODO fix this mess
        if let Some(ref producer) = app_state.kafka_outlet_producer
            && let Some(log) = log.as_str()
        {
            let status = producer
                .send(
                    FutureRecord::to("outlet_logs")
                        .payload(log.as_bytes())
                        .key(&format!("key-{id}")),
                    Duration::from_secs(0),
                )
                .await;
            if let Err((err, msg)) = status {
                error!(
                    "failed to send outlet log to kafka with its producer: err: {err}, msg: {msg:?}"
                );
            }
        }
    }

    Ok(())
}
