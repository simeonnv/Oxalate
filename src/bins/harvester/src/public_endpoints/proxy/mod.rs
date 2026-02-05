use axum::{Router, extract::DefaultBodyLimit, middleware::from_fn_with_state, routing::post};

pub mod post_proxy;
pub use post_proxy::post_proxy;

use crate::{AppState, middleware};

pub fn proxy(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(post_proxy))
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024)) // 20 mb
        .layer(from_fn_with_state(
            state.to_owned(),
            middleware::proxy_id_middleware,
        ))
}
