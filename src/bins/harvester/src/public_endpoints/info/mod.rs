use axum::Router;
use axum::routing::{get, post};

use crate::AppState;

pub mod get_uptime;
pub use get_uptime::get_uptime;

pub mod post_logs;
pub use post_logs::post_logs;

pub mod post_resources;
pub use post_resources::post_resources;

pub fn info() -> Router<AppState> {
    Router::new()
        .route("/uptime", get(get_uptime))
        .route("/logs", post(post_logs))
        .route("/resources", post(post_resources))
}
