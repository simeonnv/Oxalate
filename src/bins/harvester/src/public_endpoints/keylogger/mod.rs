// mod.rs or lib.rs
pub mod post_keylogger;
use axum::Router;
use axum::routing::post;
pub use post_keylogger::post_keylogger;

use crate::AppState;

pub fn keylogger() -> Router<AppState> {
    Router::new().route("/", post(post_keylogger))
}
