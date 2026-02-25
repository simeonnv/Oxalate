use std::ops::Deref;

use axum::{Json, extract::State};
use exn::ResultExt;
use futures::{FutureExt, future::select_all};
use http_error::HttpError;
use neo4rs::query;
use oxalate_schemas::union::post_search_graph::*;

use crate::AppState;
use itertools::Itertools;

#[utoipa::path(
    post,
    path = "/search_graph",
    request_body = Req,
    responses(
        (status = 200),
    ),
    tag = "Graph",
)]
#[axum::debug_handler]
pub async fn post_search_graph(
    State(state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<Json<Res>, HttpError> {
    let keywords = req.search_keywords;
    let recursion_depth = req.recursion_depth;

    todo!()
}

// #[derive(thiserror::Error, Debug)]
// pub enum Error {
//     #[error("failed to start the neo4j transaction")]
//     StartTxn,
//     #[error("failed to run the queries in the neo4j transaction")]
//     RunQueries,
//     #[error("failed to commit the neo4j transaction")]
//     Commit,
// }
