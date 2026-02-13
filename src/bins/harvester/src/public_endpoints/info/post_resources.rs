use std::f32;

use axum::{Extension, Json, extract::State};
use oxalate_scraper_controller::ProxyId;
use uuid::Uuid;

use oxalate_schemas::harvester::public::info::post_resources::*;

use crate::AppState;
use http_error::HttpError;

use exn::ResultExt;

#[utoipa::path(
    post,
    path = "/info/resources",
    request_body = Req,
    description = "
        Proxies shall constantly send over their resourse usage for it do be saved
        There is a good crate for collecting system usage: heim
    ",
    responses(),
    params(
      ("machine-id" = String, Header, description = "Device id"),
    ),
    tag = "Info",
)]
#[axum::debug_handler]
pub async fn post_resources(
    Extension(proxy_id): Extension<ProxyId>,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), HttpError> {
    let id = Uuid::new_v4();

    sqlx::query!(
        "
            INSERT INTO ProxyResourseUsage
                (id, device_machine_id, ram_usage, cpu_usage, net_usage_bytes)
            VALUES
                ($1, $2, $3, $4, $5); 
        ",
        id,
        proxy_id.as_ref(),
        req.ram_usage,
        req.cpu_usage,
        req.net_usage_bytes as i64,
    )
    .execute(&app_state.db_pool)
    .await
    .or_raise(|| HttpError::Internal("".into()))?;

    Ok(())
}
