use crate::Error as HttpError;
use axum::{
    debug_middleware,
    extract::{MatchedPath, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use exn::ResultExt;
use oxalate_scraper_controller::ProxyId;
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;

#[derive(Clone, Serialize)]
pub struct LoggingCTX {
    pub path: Box<str>,
    pub req_id: Uuid,
    pub proxy_id: Option<ProxyId>,
}

#[debug_middleware]
pub async fn logging_ctx_generate_middleware(
    State(_state): State<AppState>,
    matched_path: Option<MatchedPath>,
    _headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, HttpError> {
    let req_id = Uuid::new_v4();
    let path = matched_path
        .map(|e| e.as_str().to_owned())
        .unwrap_or_else(|| "404".to_string())
        .into_boxed_str();

    let proxy_id = request.extensions().get::<ProxyId>().cloned();
    let logging_ctx = LoggingCTX {
        path,
        req_id,
        proxy_id,
    };

    request.extensions_mut().insert(logging_ctx);

    let response = next.run(request).await;

    Ok(response)
}
