use std::f32;

use axum::{Json, extract::State, http::HeaderMap};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, Error};

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
    ",
    responses(),
    params(
      ("device-id" = String, Header, description = "Device id"),
    ),
    tag = "Info",
)]
#[axum::debug_handler]
pub async fn post_resources(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), Error> {
    let device_id = headers.get("device-id").and_then(|v| v.to_str().ok());
    let device_id = match device_id {
        Some(e) => e,
        None => return Err(Error::BadRequest("no device-id header!".into())),
    };

    let id = Uuid::new_v4();

    sqlx::query!(
        "
            INSERT INTO ProxyResourseUsage
                (proxy_resourse_usage_id, device_id, ram_usage, cpu_usage, net_usage_bytes)
            VALUES
                ($1, $2, $3, $4, $5); 
        ",
        id,
        device_id,
        req.ram_usage,
        req.cpu_usage,
        req.net_usage_bytes as i64,
    )
    .execute(&app_state.db_pool)
    .await?;

    Ok(())
}
