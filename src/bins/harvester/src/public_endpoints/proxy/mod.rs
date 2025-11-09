use axum::{Router, routing::any};

pub mod ws_proxy;
pub use ws_proxy::ws_proxy;

use crate::AppState;

pub fn proxy() -> Router<AppState> {
    Router::new().route("/", any(ws_proxy))
}
