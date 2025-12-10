use crate::{AppState, Error};
use axum::{Json, extract::State, http::HeaderMap};

use oxalate_schemas::harvester::public::proxy::post_proxy::*;
use oxalate_scrapper_controller::ProxyId;

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
    let proxy_id = ProxyId::from_http_headers(&headers, &app_state.db_pool).await?;
    dbg!(&req);

    match req {
        Req::RequestUrls => {
            let proxy_job = app_state.scrapper_state.get_job(&proxy_id);
            dbg!(&proxy_job);
            // TODO fix this fucking copy
            Ok(Json(Res(proxy_job.map(|e| e.reqs.clone()))))
        }
        Req::ReturnUrlOutputs(proxy_outputs) => {
            app_state
                .scrapper_state
                .complete_job(&proxy_id, &proxy_outputs, app_state.db_pool)
                .await?;

            Ok(Json(Res(None)))
        }
    }
}
