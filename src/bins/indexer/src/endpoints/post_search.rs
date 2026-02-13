use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Req {}

#[utoipa::path(
    post,
    path = "/search",
    responses(
        (status = 200),
    ),
    tag = "Search",
)]
#[axum::debug_handler]
pub async fn post_search(Json(req): Json<Req>) -> &'static str {
    "pong"
}
