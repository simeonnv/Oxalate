use crate::{AppState, Error, insure_device_exists};
use axum::{extract::State, http::HeaderMap};

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
) -> Result<(), Error> {
    let machine_id = headers.get("machine-id").and_then(|v| v.to_str().ok());
    let machine_id = match machine_id {
        Some(e) => e.to_owned(),
        None => return Err(Error::BadRequest("no or invalid machine id!".into())),
    };
    insure_device_exists(&machine_id, &app_state.db_pool).await?;

    sqlx::query!(
        "
            INSERT INTO Uptime
                (device_machine_id)
            VALUES
                ($1)
            ;
        ",
        machine_id
    )
    .execute(&app_state.db_pool)
    .await?;

    Ok(())
}
