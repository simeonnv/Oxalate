use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{AppState, endpoints::api_docs::ApiDoc};

pub mod get_ping;
use get_ping::get_ping;

pub mod post_search;
use post_search::post_search;

mod api_docs;

pub mod post_keyword_graph;
use post_keyword_graph::post_keyword_graph;

pub fn endpoints(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .route("/search", post(post_search))
        .route("/keyword_graph", post(post_keyword_graph))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
    // .route(
    //     "/swagger",
    //     get(|| async { Redirect::permanent("/swagger/") }),
    // )
}
