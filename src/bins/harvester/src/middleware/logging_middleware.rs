use std::sync::{Arc, RwLock};

use crate::Error as HttpError;
use axum::{
    debug_middleware,
    extract::{MatchedPath, Request, State},
    http::{HeaderMap, Method, Uri},
    middleware::Next,
    response::Response,
};
use oxalate_scraper_controller::ProxyId;
use serde::{Serialize, Serializer};
use uuid::Uuid;

use crate::AppState;

#[derive(Clone)]
pub struct LoggingCTX(Arc<RwLock<LoggingCTXInner>>);
impl Serialize for LoggingCTX {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = self.0.read().map_err(serde::ser::Error::custom)?;
        data.serialize(serializer)
    }
}
impl LoggingCTX {
    pub fn with_mutate<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut LoggingCTXInner) -> R,
    {
        let mut guard = self.0.write().unwrap();
        f(&mut guard)
    }
}

#[derive(Clone, Serialize)]
pub struct LoggingCTXInner {
    pub path: Box<str>,
    pub req_id: Uuid,

    pub proxy_id: Option<ProxyId>,

    pub method: Box<str>,

    pub uri: Box<str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

impl LoggingCTX {
    pub fn new(method: Method, uri: Uri, endpoint_path: Option<MatchedPath>) -> Self {
        let req_id = Uuid::new_v4();
        let method = method.as_str().to_owned().into_boxed_str();
        let uri = uri.to_string().into_boxed_str();
        let path = endpoint_path
            .map(|e| e.as_str().to_owned())
            .unwrap_or_else(|| "404".to_string())
            .into_boxed_str();

        Self(Arc::new(RwLock::new(LoggingCTXInner {
            path,
            req_id,
            proxy_id: None,
            method,
            uri,
            status: None,
        })))
    }
}

#[debug_middleware]
pub async fn logging_middleware(
    State(_state): State<AppState>,
    matched_path: Option<MatchedPath>,
    _headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, HttpError> {
    let logging_ctx = LoggingCTX::new(
        request.method().to_owned(),
        request.uri().to_owned(),
        matched_path,
    );
    request.extensions_mut().insert(logging_ctx.to_owned());

    let response = next.run(request).await;

    log::info!(ctx:serde = logging_ctx; "request start");
    let status = response.status().as_u16();
    logging_ctx.with_mutate(|e| e.status = Some(status));
    log::info!(ctx:serde = logging_ctx; "request end");

    Ok(response)
}
