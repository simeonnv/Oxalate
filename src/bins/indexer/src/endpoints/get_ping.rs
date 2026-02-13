#[utoipa::path(
    get,
    path = "/ping",
    responses(
        (status = 200),
    ),
    tag = "Health",
)]
#[axum::debug_handler]
pub async fn get_ping() -> &'static str {
    "pong"
}
