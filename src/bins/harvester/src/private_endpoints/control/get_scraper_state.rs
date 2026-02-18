use axum::{Json, extract::State};
use oxalate_schemas::harvester::private::control::get_scraper_state::Res;

use crate::AppState;

#[utoipa::path(
    get,
    path = "/control/scraper_state",
    responses(
        (status = 200),
    ),
    description = "",
    tag = "Control",
)]
pub async fn get_scraper_state(State(app_state): State<AppState>) -> Json<Res> {
    let res = Res {
        enabled: app_state
            .scraper_controller
            .enabled
            .load(std::sync::atomic::Ordering::Relaxed),
    };

    Json(res)
}
