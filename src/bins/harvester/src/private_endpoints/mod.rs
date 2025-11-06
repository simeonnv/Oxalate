use axum::{Router, routing::get};

mod get_ping;
pub use get_ping::get_ping;

use crate::AppState;

pub fn private_endpoints() -> Router<AppState> {
    Router::new().route("/ping", get(get_ping))
}
