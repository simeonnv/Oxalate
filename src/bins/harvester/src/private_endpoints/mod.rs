use axum::{Router, routing::get};

mod get_ping;
pub use get_ping::get_ping;

use crate::AppState;

#[utoipa::path(
    post,
    path = "/ping",
    responses(
        (status = 200),
    ),
    tag = "Health",
)]
pub fn private_endpoints() -> Router<AppState> {
    Router::new().route("/ping", get(get_ping))
}
