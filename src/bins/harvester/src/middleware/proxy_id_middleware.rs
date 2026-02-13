use crate::middleware::logging_middleware::LoggingCTX;
use axum::{
    debug_middleware,
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use exn::ResultExt;
use http_error::HttpError;
use oxalate_scraper_controller::ProxyId;

use crate::AppState;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to extract proxy id in middleware")]
    ProxyId,
}

#[debug_middleware]
pub async fn proxy_id_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, HttpError> {
    let proxy_id = ProxyId::from_http_headers(&headers, &state.db_pool)
        .await
        .or_raise(|| Error::ProxyId)
        .or_raise(|| HttpError::BadRequest("No proxy id in header!".into()))?;

    let ext = request.extensions_mut();
    ext.insert(proxy_id.to_owned());
    if let Some(e) = ext.get_mut::<LoggingCTX>() {
        e.with_mutate(|e| e.proxy_id = Some(proxy_id));
    }

    let response = next.run(request).await;

    Ok(response)
}
