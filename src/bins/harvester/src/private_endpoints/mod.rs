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
pub fn private_endpoints(_state: &AppState) -> Router<AppState> {
    Router::new().route("/ping", get(get_ping))
}
