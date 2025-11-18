use std::f32;

use axum::{Json, extract::State, http::HeaderMap};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, Error, insure_device_exists};

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::Info::Resources::Req)]
pub struct Req {
    pub ram_usage: f32,
    pub cpu_usage: f32,
    pub net_usage_bytes: u32,
}

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
    let id = Uuid::new_v4();

    sqlx::query!(
        "
            INSERT INTO ProxyResourseUsage
                (id, device_machine_id, ram_usage, cpu_usage, net_usage_bytes)
            VALUES
                ($1, $2, $3, $4, $5); 
        ",
        id,
        machine_id,
        req.ram_usage,
        req.cpu_usage,
        req.net_usage_bytes as i64,
    )
    .execute(&app_state.db_pool)
    .await?;

    Ok(())
}
