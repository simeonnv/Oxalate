use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};

use crate::AppState;
use crate::middleware::proxy_id_middleware;

pub mod post_logs;
pub use post_logs::post_logs;

pub mod post_resources;
pub use post_resources::post_resources;

pub mod get_uptime;
pub use get_uptime::get_uptime;

pub fn info(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/uptime", get(get_uptime))
        .route("/logs", post(post_logs))
        .route("/resources", post(post_resources))
        .layer(from_fn_with_state(state.to_owned(), proxy_id_middleware))
}
