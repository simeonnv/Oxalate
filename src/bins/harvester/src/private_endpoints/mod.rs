use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod get_ping;
pub use get_ping::get_ping;

mod api_docs;
pub use api_docs::ApiDoc;

mod metric;

mod control;

use crate::{AppState, private_endpoints::control::control};

pub fn private_endpoints(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .nest("/control", control(_state))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
