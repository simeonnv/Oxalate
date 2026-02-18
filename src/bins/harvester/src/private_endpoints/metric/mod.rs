use crate::AppState;
use axum::{Router, routing::get};

pub mod get_active_tasks;
pub use get_active_tasks::get_active_tasks;

pub mod get_connected_proxies;

pub mod get_file_iterator_task_generator_state;

pub fn metric(_state: &AppState) -> Router<AppState> {
    Router::new().route("/active_tasks", get(get_active_tasks))
    // .route("/swap_scraper_on_state", post(post_swap_scraper_on_state))
}
