#[utoipa::path(
    get,
    path = "/ping",
    responses(
        (status = 200),
    ),
    tag = "Health",
)]
pub async fn get_ping() -> &'static str {
    "pong"
}
