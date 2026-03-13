use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    debug_middleware,
    extract::{ConnectInfo, MatchedPath, Request},
    http::{HeaderMap, Method, Uri},
    middleware::Next,
    response::Response,
};
use http_error::HttpError;
use serde::{Serialize, Serializer};
use uuid::Uuid;

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

    pub fn add_extra<T: Serialize>(&self, key: &str, value: T) {
        if let Ok(val) = serde_json::to_value(value) {
            self.with_mutate(|inner| {
                inner.extra_ctx.insert(key.to_string(), val);
            });
        }
    }
}

#[derive(Clone, Serialize)]
pub struct LoggingCTXInner {
    pub path: Box<str>,
    pub req_id: Uuid,

    pub method: Box<str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    pub ip: SocketAddr,

    pub uri: Box<str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,

    pub extra_ctx: serde_json::Map<String, serde_json::Value>,
}

impl LoggingCTX {
    pub fn new(
        method: Method,
        uri: Uri,
        endpoint_path: Option<MatchedPath>,
        host: Option<String>,
        ip: SocketAddr,
    ) -> Self {
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
            method,
            uri,
            status: None,
            host,
            ip,
            extra_ctx: serde_json::Map::new(),
        })))
    }
}

#[debug_middleware]
pub async fn logging_middleware(
    matched_path: Option<MatchedPath>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, HttpError> {
    let host = request
        .headers()
        .get("host")
        .and_then(|e| e.to_str().ok())
        .map(|e| e.to_owned());

    let logging_ctx = LoggingCTX::new(
        request.method().to_owned(),
        request.uri().to_owned(),
        matched_path,
        host,
        addr,
    );
    request.extensions_mut().insert(logging_ctx.to_owned());
    log::debug!(ctx:serde = logging_ctx; "request start");

    let response = next.run(request).await;

    logging_ctx.with_mutate(|e| e.status = Some(response.status().as_u16()));
    log::debug!(ctx:serde = logging_ctx; "request end");

    Ok(response)
}
