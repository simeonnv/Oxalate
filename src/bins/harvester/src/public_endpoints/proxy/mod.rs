use axum::{Router, routing::post};

pub mod post_proxy;
pub use post_proxy::post_proxy;

use crate::AppState;

pub fn proxy() -> Router<AppState> {
    Router::new().route("/", post(post_proxy))
}
