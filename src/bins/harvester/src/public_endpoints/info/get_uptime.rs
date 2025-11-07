use crate::{AppState, Error};
use axum::{extract::State, http::HeaderMap};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/info/uptime",
    description = "devices will ping this endpoint for health/uptime monitoring",
    params(
      ("device-id" = String, Header, description = "Device id"),
    ),
    tag = "Info",
    responses()
)]
#[axum::debug_handler]
pub async fn get_uptime(
    headers: HeaderMap,
    State(app_state): State<AppState>,
) -> Result<(), Error> {
    let device_id = headers
        .get("device-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let uptime_id = Uuid::new_v4();
    sqlx::query!(
        r#"
            INSERT INTO Uptime
                (uptime_id, device_id)
                VALUES ($1, $2)
            ;
        "#,
        uptime_id,
        device_id
    )
    .execute(&app_state.db_pool)
    .await?;
    Ok(())
}
