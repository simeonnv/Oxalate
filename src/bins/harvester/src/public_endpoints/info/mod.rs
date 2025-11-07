use axum::Router;
use axum::routing::get;

use crate::AppState;

pub mod get_uptime;
pub use get_uptime::get_uptime;

pub fn info() -> Router<AppState> {
    Router::new().route("/uptime", get(get_uptime))
}
