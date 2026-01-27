use std::ops::Deref;

use crate::{AppState, Error as HttpError, middleware::logging_middleware::LoggingCTX};
use axum::{Extension, Json, extract::State};
use exn::ResultExt;
use oxalate_schemas::harvester::public::proxy::post_proxy::*;
use oxalate_scraper_controller::ProxyId;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to return urls after a request urls request")]
    ReqUrls,
    #[error("failed to handle the urls after a return url outputs request")]
    ReturnUrls,
}

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
    Extension(logging_ctx): Extension<LoggingCTX>,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<Json<Res>, HttpError> {
    match req {
        Req::RequestUrls => {
            let proxy_job = app_state
                .scraper_controller
                .get_task(&proxy_id, (), &logging_ctx)
                .await
                .or_raise(|| Error::ReqUrls)
                .or_raise(|| HttpError::Internal("".into()))?;

            Ok(Json(Res(proxy_job.map(|e| e.deref().clone()))))
        }
        Req::ReturnUrlOutputs(proxy_outputs) => {
            app_state
                .scraper_controller
                .complete_task(&proxy_id, &proxy_outputs, &app_state.db_pool, &logging_ctx)
                .await
                .or_raise(|| Error::ReturnUrls)
                .or_raise(|| HttpError::Internal("".into()))?;

            Ok(Json(Res(None)))
        }
    }
}
