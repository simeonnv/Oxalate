use axum::{
    Router,
    response::Redirect,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{AppState, endpoints::app_docs::ApiDoc};

pub mod get_ping;
use get_ping::get_ping;

pub mod post_search;
use post_search::post_search;

mod app_docs;

pub fn endpoints(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .route("/search", post(post_search))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
    // .route(
    //     "/swagger",
    //     get(|| async { Redirect::permanent("/swagger/") }),
    // )
}
