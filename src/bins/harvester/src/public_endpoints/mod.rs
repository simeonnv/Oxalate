use axum::{Router, routing::get};

mod get_ping;
pub use get_ping::get_ping;

pub mod keylogger;
pub use keylogger::keylogger;

use crate::AppState;

pub fn public_endpoints() -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .nest("/keylogger", keylogger())
}
