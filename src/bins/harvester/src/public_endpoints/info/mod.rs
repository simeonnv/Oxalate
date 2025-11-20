use axum::Router;
use axum::routing::{any, post};

use crate::AppState;

pub mod post_logs;
pub use post_logs::post_logs;

pub mod post_resources;
pub use post_resources::post_resources;

pub mod ws_uptime;
pub use ws_uptime::ws_uptime;

pub fn info() -> Router<AppState> {
    Router::new()
        .route("/uptime", any(ws_uptime))
        .route("/logs", post(post_logs))
        .route("/resources", post(post_resources))
}
