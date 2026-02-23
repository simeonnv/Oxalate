use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{AppState, endpoints::api_docs::ApiDoc};
pub mod api_docs;

pub mod get_ping;
use get_ping::get_ping;

pub mod post_insert_keywords;
pub use post_insert_keywords::post_insert_keywords;

pub fn endpoints(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .route("/insert_keywords", post(post_insert_keywords))
        // .route("/search", post(post_search))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
