use axum::{Json, extract::State, http::HeaderMap};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, Error};

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::Info::Logs::Req)]
pub struct Req {
    pub logs: Vec<Log>,
}

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::Info::Logs::Req::Log)]
pub struct Log {
    pub log_level: String,
    pub body: String,
}

#[utoipa::path(
    post,
    path = "/info/logs",
    request_body = Req,
    responses(),
    params(
      ("device-id" = String, Header, description = "Device id"),
    ),
    tag = "Info",
)]
#[axum::debug_handler]
pub async fn post_logs(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), Error> {
    let device_id = headers.get("device-id").and_then(|v| v.to_str().ok());
    let device_id = match device_id {
        Some(e) => e,
        None => return Err(Error::BadRequest("no device-id header!".into())),
    };

    async_scoped::TokioScope::scope_and_block(|spawner| {
        for log in &req.logs {
            let db_pool = app_state.db_pool.clone();
            spawner.spawn(async move {
                let id = Uuid::new_v4();
                let _ = sqlx::query!(
                    "
                        INSERT INTO Logs
                            (log_id, log_level, body, device_id)
                        VALUES ($1, $2, $3, $4);
                    ",
                    id,
                    log.log_level,
                    log.body,
                    device_id,
                )
                .execute(&db_pool)
                .await;
            });
        }
    });

    Ok(())
}
