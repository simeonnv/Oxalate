use crate::{AppState, middleware::logging_middleware::LoggingCTX};
use axum::{Extension, Json, debug_handler, extract::State};
use oxalate_schemas::harvester::private::metric::get_connected_proxies::*;

#[utoipa::path(
    get,
    path = "/metric/connected_proxies",
    responses(
        (status = 200),
    ),
    description = "",
    tag = "Metric",
)]
#[debug_handler]
pub async fn get_connected_proxies(
    State(state): State<AppState>,
    Extension(logging_ctx): Extension<LoggingCTX>,
) -> Json<Res> {
    let mut connected_proxies = Vec::with_capacity(state.proxy_connection_store.inner.len());

    dbg!(&state.proxy_connection_store);
    log::debug!(ctx:serde = logging_ctx; "iterating over proxy connection store");
    for kv in state.proxy_connection_store.inner.iter() {
        let (k, (_, rx)) = kv.pair();
        if *rx.borrow() {
            connected_proxies.push(k.to_owned());
        }
    }
    log::debug!(ctx:serde = logging_ctx; "fully iterated over proxy connection store");

    Json(Res { connected_proxies })
}
