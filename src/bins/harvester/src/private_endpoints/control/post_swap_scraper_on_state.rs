use std::sync::atomic::Ordering;

use axum::extract::State;

use crate::AppState;

#[utoipa::path(
    post,
    path = "/control/swap_scraper_on_state",
    responses(
        (status = 200),
    ),
    description = "",
    tag = "Control",
)]
pub async fn post_swap_scraper_on_state(State(app_state): State<AppState>) {
    app_state
        .scraper_controller
        .enabled
        .fetch_not(Ordering::Relaxed);
}
