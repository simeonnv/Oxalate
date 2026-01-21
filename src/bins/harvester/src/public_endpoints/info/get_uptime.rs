use crate::{AppState, Error};
use axum::{extract::State, http::HeaderMap};
use exn::ResultExt;
use oxalate_scraper_controller::ProxyId;

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
    let proxy_id = ProxyId::from_http_headers(&headers, &app_state.db_pool)
        .await
        .or_raise(|| Error::BadRequest("No proxy id in header".into()))?;

    sqlx::query!(
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
    .or_raise(|| Error::Internal("".into()))?;

    Ok(())
}
