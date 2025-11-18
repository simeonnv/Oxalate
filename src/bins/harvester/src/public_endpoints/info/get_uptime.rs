use crate::{AppState, Error, insure_device_exists};
use axum::{extract::State, http::HeaderMap};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/info/uptime",
    description = "devices will ping this endpoint for health/uptime monitoring",
    params(
      ("machine-id" = String, Header, description = "the hardware uuid"),
    ),
    tag = "Info",
    responses()
)]
#[axum::debug_handler]
pub async fn get_uptime(
    headers: HeaderMap,
    State(app_state): State<AppState>,
) -> Result<(), Error> {
    let machine_id = headers
        .get("machine-id")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::BadRequest("no machine-id header!".into()))?;
    insure_device_exists(machine_id, &app_state.db_pool).await?;
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
            INSERT INTO Uptime
                (id, device_machine_id)
                VALUES ($1, $2)
            ;
        "#,
        id,
        machine_id
    )
    .execute(&app_state.db_pool)
    .await?;
    Ok(())
}
