use crate::AppState;
use axum::{Router, routing::get};

pub mod get_active_tasks;
use get_active_tasks::get_active_tasks;

pub mod get_connected_proxies;
use get_connected_proxies::get_connected_proxies;

pub fn metric(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/active_tasks", get(get_active_tasks))
        .route("/connected_proxies", get(get_connected_proxies))
    // .route("/swap_scraper_on_state", post(post_swap_scraper_on_state))
}
