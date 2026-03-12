use axum::{Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;
use oxalate_parsing::save_meta_webpage_into_postgres::save_meta_webpage_into_postgres;

use crate::AppState;
use oxalate_schemas::parser::post_insert_meta_webpage::*;

pub use oxalate_parsing::save_into_neo4j::save_into_neo4j;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to insert parsed html into neo4j")]
    InsertNeo4j,

    #[error("failed to insert parsed html into postges")]
    InsertPg,
}

#[utoipa::path(
    post,
    path = "/insert_meta_webpage",
    request_body = Req,
    responses(
        (status = 200),
    ),
    tag = "Insert",
)]
#[axum::debug_handler]
pub async fn post_insert_meta_webpage(
    State(state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), HttpError> {
    let pg_result = save_meta_webpage_into_postgres(
        &state.db_pool,
        &req.keywords,
        req.title,
        req.url.to_owned(),
        req.search_engine,
    )
    .await;

    let neo4j_result = save_into_neo4j(&state.neo4j_pool, &req.keywords, &req.url, 5).await;

    neo4j_result
        .or_raise(|| Error::InsertNeo4j)
        .or_raise(|| HttpError::Internal("".into()))?;
    pg_result
        .or_raise(|| Error::InsertPg)
        .or_raise(|| HttpError::Internal("".into()))?;

    Ok(())
}
