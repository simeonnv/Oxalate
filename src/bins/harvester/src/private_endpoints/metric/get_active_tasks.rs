use std::collections::HashMap;

use crate::{AppState, middleware::logging_middleware::LoggingCTX};
use axum::{Extension, Json, debug_handler, extract::State};
use oxalate_schemas::harvester::private::metric::get_active_tasks::*;

#[utoipa::path(
    get,
    path = "/metric/active_tasks",
    responses(
        (status = 200),
    ),
    description = "",
    tag = "Metric",
)]
#[debug_handler]
pub async fn get_active_tasks(
    State(state): State<AppState>,
    Extension(logging_ctx): Extension<LoggingCTX>,
) -> Json<Res> {
    let mut active_tasks_copy =
        HashMap::with_capacity(state.scraper_controller.active_tasks.len() * 2);

    log::info!(ctx:serde = logging_ctx; "locking up active tasks from scraper controller bc of cloning");
    for pair in state.scraper_controller.active_tasks.iter() {
        let (k, v) = pair.pair();
        active_tasks_copy.insert(k.to_owned(), v.to_owned());
    }
    log::info!(ctx:serde = logging_ctx; "unlocked active tasks");

    Json(Res {
        active_tasks: active_tasks_copy,
    })
}
