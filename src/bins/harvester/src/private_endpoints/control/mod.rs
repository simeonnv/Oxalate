use crate::AppState;
use axum::{Router, routing::get};

pub mod get_scraper_state;
use get_scraper_state::get_scraper_state;

mod post_swap_scraper_on_state;

pub fn control(_state: &AppState) -> Router<AppState> {
    Router::new().route("/scraper_state", get(get_scraper_state))
}
