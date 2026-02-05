use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod get_ping;
pub use get_ping::get_ping;

pub mod keylogger;
pub use keylogger::keylogger;

pub mod info;
pub use info::info;

mod api_docs;
pub use api_docs::ApiDoc;

pub mod proxy;
pub use proxy::proxy;

use crate::AppState;

pub fn public_endpoints(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .nest("/keylogger", keylogger(state))
        .nest("/info", info(state))
        .nest("/proxy", proxy(state))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
    // .route(
    //     "/swagger",
    //     get(|| async { Redirect::permanent("/swagger/") }),
    // )
}
