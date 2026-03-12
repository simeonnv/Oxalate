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

pub mod post_insert_meta_webpage;
use post_insert_meta_webpage::post_insert_meta_webpage;

pub mod post_insert_webpage;
use post_insert_webpage::post_insert_webpage;

pub fn endpoints(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .route("/insert_meta_webpage", post(post_insert_meta_webpage))
        .route("/insert_webpage", post(post_insert_webpage))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
