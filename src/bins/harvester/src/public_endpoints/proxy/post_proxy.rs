use crate::{AppState, Error};
use axum::{Extension, Json, extract::State, http::HeaderMap};

use oxalate_schemas::harvester::public::proxy::post_proxy::*;
use oxalate_scraper_controller::ProxyId;

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
    Extension(proxy_id): Extension<ProxyId>,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<Json<Res>, Error> {
    match req {
        Req::RequestUrls => {
            let proxy_job = app_state.scraper_controller.get_job(&proxy_id);
            // TODO fix this fucking copy
            Ok(Json(Res(proxy_job.map(|e| e.reqs.clone()))))
        }
        Req::ReturnUrlOutputs(proxy_outputs) => {
            app_state
                .scraper_controller
                .complete_task(&proxy_id, &proxy_outputs, &app_state.db_pool, &())
                .await?;

            Ok(Json(Res(None)))
        }
    }
}
