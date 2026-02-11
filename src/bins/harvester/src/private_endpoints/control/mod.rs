use crate::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub mod get_scraper_state;
use get_scraper_state::get_scraper_state;

pub mod post_swap_scraper_on_state;
use post_swap_scraper_on_state::post_swap_scraper_on_state;

pub fn control(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/scraper_state", get(get_scraper_state))
        .route("/swap_scraper_on_state", post(post_swap_scraper_on_state))
}
