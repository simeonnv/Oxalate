use axum::{Json, extract::State, http::HeaderMap};
use exn::ResultExt;
use serde::Serialize;

use crate::{AppState, Error as HttpError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to parse scraper controller to json")]
    JsonParse,
}

#[derive(Serialize)]
pub struct Res {
    pub enabled: bool,
}

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
