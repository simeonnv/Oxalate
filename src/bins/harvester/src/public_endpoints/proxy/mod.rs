use axum::{Router, extract::DefaultBodyLimit, routing::post};

pub mod post_proxy;
pub use post_proxy::post_proxy;

use crate::AppState;

pub fn proxy() -> Router<AppState> {
    Router::new()
        .route("/", post(post_proxy))
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024)) // 20 mb
}
