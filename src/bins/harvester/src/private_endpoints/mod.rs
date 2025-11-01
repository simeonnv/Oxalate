use axum::{Router, routing::get};

mod get_ping;
pub use get_ping::get_ping;

pub fn private_endpoints() -> Router {
    Router::new().route("/ping", get(get_ping))
}
