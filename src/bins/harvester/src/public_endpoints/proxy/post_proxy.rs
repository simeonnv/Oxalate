use std::ops::Deref;

use crate::{AppState, proxy_settings_store::TaskGenerators};
use axum::{Extension, Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;
use log::info;
use oxalate_middleware::logging_middleware::LoggingCTX;
use oxalate_schemas::harvester::public::proxy::post_proxy::*;
use oxalate_scraper_controller::{ProxyId, scraper_controller::ProxyRes};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to return urls after a request urls request")]
    ReqUrls,

    #[error("failed to handle the urls after a return url outputs request")]
    ReturnUrls,

    #[error("failed to send page to parser")]
    SendParser,

    #[error("Parser returned a error on page after sending pages")]
    ErrorParser,
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
    let proxy_settings = app_state
        .proxy_settings_store
        .get_or_create_settings(proxy_id.clone());

    match req {
        Req::RequestUrls => {
            info!(ctx:serde = logging_ctx; "requested proxy job, creating one");

            let proxy_job = app_state.scraper_controller;
            let proxy_job = match proxy_settings.task_generator {
                TaskGenerators::FileIteratorTaskGenerator(file_iterator_task_generator) => {
                    proxy_job
                        .get_task(
                            &proxy_id,
                            file_iterator_task_generator.as_ref(),
                            &logging_ctx,
                        )
                        .await
                }
            }
            .or_raise(|| Error::ReqUrls)
            .or_raise(|| HttpError::Internal("".into()))?;

            Ok(Json(Res(proxy_job.map(|e| e.deref().clone()))))
        }
        Req::ReturnUrlOutputs(proxy_outputs) => {
            info!(ctx:serde = logging_ctx; "proxy is returning job outputs, handling task");

            use oxalate_schemas::parser::post_insert_webpage::{Page, Req};

            app_state
                .scraper_controller
                .mark_task_as_complete(&proxy_id, &proxy_outputs, &logging_ctx)
                .await
                .or_raise(|| Error::ReturnUrls)
                .or_raise(|| HttpError::Internal("".into()))?;

            let pages = proxy_outputs
                .into_iter()
                .map(|e| match e {
                    ProxyRes::HttpRes(http_res) => Page {
                        url: http_res.url,
                        raw_html: http_res.body,
                        headers: Some(http_res.headers),
                        proxy_id: proxy_id.to_owned(),
                    },
                })
                .collect();

            app_state
                .reqwest_client
                .post(app_state.parser_url.join("insert_meta_webpage").unwrap())
                .json(&Req { pages })
                .send()
                .await
                .or_raise(|| Error::SendParser)
                .or_raise(|| HttpError::Internal("".into()))?
                .error_for_status()
                .or_raise(|| Error::ErrorParser)
                .or_raise(|| HttpError::Internal("".into()))?;

            Ok(Json(Res(None)))
        }
    }
}
