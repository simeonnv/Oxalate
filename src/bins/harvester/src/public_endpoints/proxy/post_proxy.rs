use crate::{AppState, Error, insure_device_exists};
use axum::{Json, extract::State, http::HeaderMap};

use oxalate_schemas::harvester::public::proxy::post_proxy::*;

#[utoipa::path(
    post,
    path = "/proxy",
    responses(
        (status = 200),
    ),
    description = "",
    params(
      ("machine-id" = String, Header, description = "Device hardware id"),
    ),
    tag = "Proxy",
)]
#[axum::debug_handler]
pub async fn post_proxy(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<Json<Res>, Error> {
    let machine_id = headers.get("machine-id").and_then(|v| v.to_str().ok());
    let machine_id = match machine_id {
        Some(e) => e,
        None => return Err(Error::BadRequest("no or invalid device id!".into())),
    };
    insure_device_exists(machine_id, &app_state.db_pool).await?;

    match req {
        Req::RequestUrls => {
            let proxy_job = app_state.scrapper_state.req_addresses(machine_id);
            Ok(Json(Res(proxy_job.map(|e| e.urls.clone()))))
        }
        Req::ReturnUrlOutputs(proxy_outputs) => {
            app_state
                .scrapper_state
                .complete_job(machine_id, &proxy_outputs, app_state.db_pool)
                .await?;

            Ok(Json(Res(None)))
        }
    }
}
